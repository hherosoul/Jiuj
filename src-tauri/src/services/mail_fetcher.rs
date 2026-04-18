use crate::models::RawEmail;
use crate::db::AccountRepo;
use crate::db::SkipListRepo;
use crate::services::sanitizer::Sanitizer;
use crate::services::secret_store::SecretStore;
use chrono::Datelike;
use native_tls::TlsConnector;
use uuid::Uuid;

#[derive(Clone)]
pub struct MailFetcher {
    account_repo: AccountRepo,
    secret_store: SecretStore,
    sanitizer: Sanitizer,
}

impl MailFetcher {
    pub fn new(account_repo: AccountRepo, secret_store: SecretStore, sanitizer: Sanitizer) -> Self {
        MailFetcher {
            account_repo,
            secret_store,
            sanitizer,
        }
    }

    pub fn test_connection(&self, email: &str, host: &str, port: u16, password: &str) -> Result<(), Box<dyn std::error::Error>> {
        let tls = TlsConnector::builder().build()?;
        let client = imap::connect((host, port), host, &tls)?;
        let mut session = client.login(email, password).map_err(|e| e.0)?;
        session.select("INBOX")?;
        session.logout()?;
        Ok(())
    }

    pub fn fetch_emails(&self, skip_list_repo: &SkipListRepo) -> Result<Vec<RawEmail>, Box<dyn std::error::Error>> {
        let accounts = self.account_repo.get_all()?;
        let mut all_emails = Vec::new();

        for account in accounts {
            if account.status == crate::models::AccountStatus::Disabled {
                continue;
            }

            let password_key = format!("password:{}", account.id);
            let password = match self.secret_store.get_password(&password_key, &crate::db::SettingsRepo::new(self.account_repo.db.clone()))? {
                Some(p) => p,
                None => continue,
            };

            if let Err(e) = self.fetch_account_emails(&account, &password, skip_list_repo, &mut all_emails) {
                log::error!("Failed to fetch emails for {}: {}", account.email, e);
            }
        }

        Ok(all_emails)
    }

    fn extract_from_address(header_bytes: &[u8]) -> Option<String> {
        let parsed = mail_parser::MessageParser::default().parse(header_bytes);
        parsed.and_then(|msg| {
            msg.from().and_then(|from| {
                from.first().and_then(|a| a.address()).map(|s| s.to_string())
            })
        })
    }

    fn fetch_account_emails(
        &self,
        account: &crate::models::Account,
        password: &str,
        skip_list_repo: &SkipListRepo,
        all_emails: &mut Vec<RawEmail>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("[MailFetcher] Connecting to {}:{} for {}", account.imap_host, account.imap_port, account.email);
        let tls = TlsConnector::builder().build()?;
        let client = imap::connect((account.imap_host.as_str(), account.imap_port), &account.imap_host, &tls)?;
        log::info!("[MailFetcher] Connected, logging in...");
        let mut session = client.login(&account.email, password).map_err(|e| e.0)?;
        log::info!("[MailFetcher] Logged in, selecting INBOX...");
        session.select("INBOX")?;
        log::info!("[MailFetcher] INBOX selected, building search query...");

        let search_query = if account.last_uid > 0 {
            format!("UID {}:*", account.last_uid + 1)
        } else {
            let since_date = chrono::Utc::now() - chrono::Duration::days(3);
            let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
            let since_str = format!("{:02}-{}-{}", since_date.day(), months[since_date.month() as usize - 1], since_date.year());
            format!("SINCE {}", since_str)
        };

        log::info!("[MailFetcher] Search query: {} (last_uid={})", search_query, account.last_uid);

        let uids = session.search(&search_query)?;
        log::info!("[MailFetcher] Found {} UIDs", uids.len());

        if uids.is_empty() {
            session.logout()?;
            return Ok(());
        }

        let mut max_uid = account.last_uid;
        let mut new_count = 0;
        let mut skipped_count = 0;

        for &uid in uids.iter() {
            let uid_u64 = uid as u64;
            if uid_u64 > max_uid {
                max_uid = uid_u64;
            }

            let header_messages = session.fetch(
                uid.to_string(),
                "(BODY.PEEK[HEADER.FIELDS (FROM)])",
            )?;

            let mut should_skip = false;
            for msg in header_messages.iter() {
                if let Some(header_bytes) = msg.body() {
                    if let Some(from_addr) = Self::extract_from_address(header_bytes) {
                        if skip_list_repo.is_skipped(&from_addr)? {
                            log::debug!("[MailFetcher] Skipping UID {} from skipped sender: {}", uid, from_addr);
                            should_skip = true;
                            skipped_count += 1;
                        }
                    }
                }
            }

            if should_skip {
                continue;
            }

            log::debug!("[MailFetcher] Fetching full message for UID {}", uid);

            let messages = session.fetch(uid.to_string(), "(RFC822)")?;
            for message in messages.iter() {
                if let Some(body) = message.body() {
                    let parsed = mail_parser::MessageParser::default().parse(body);
                    if let Some(parsed) = parsed {
                        if let Some(from) = parsed.from() {
                            if let Some(from_addr) = from.first().and_then(|a| a.address()) {
                                let subject = parsed.subject().unwrap_or("").to_string();
                                let date = parsed.date().map(|d| d.to_rfc3339()).unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

                                let body_text = parsed.body_text(0).map(|c| c.to_string()).unwrap_or_default();
                                let (processed, truncated) = self.sanitizer.process(&body_text);

                                let raw_email = RawEmail {
                                    id: Uuid::new_v4().to_string(),
                                    from: from_addr.to_string(),
                                    subject,
                                    date,
                                    body: processed,
                                    account_id: account.id.clone(),
                                    truncated,
                                };

                                all_emails.push(raw_email);
                                new_count += 1;
                            }
                        }
                    }
                }
            }
        }

        log::info!("[MailFetcher] Finished: {} new, {} skipped from {}", new_count, skipped_count, account.email);
        if max_uid > account.last_uid {
            self.account_repo.update_last_uid(&account.id, max_uid)?;
        }

        session.logout()?;
        Ok(())
    }
}
