use crate::constants::*;
use crate::db::*;
use crate::services::*;
use crate::skills::SkillLoader;
use tokio::time::{interval, Duration as TokioDuration};
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

pub struct AppScheduler {
    item_repo: ItemRepo,
    skill_repo: SkillRepo,
    account_repo: AccountRepo,
    skip_list_repo: SkipListRepo,
    settings_repo: SettingsRepo,
    mail_fetcher: MailFetcher,
    ai_analyzer: AIAnalyzer,
    reminder_engine: ReminderEngine,
}

impl AppScheduler {
    pub fn new(
        item_repo: ItemRepo,
        account_repo: AccountRepo,
        skill_repo: SkillRepo,
        skip_list_repo: SkipListRepo,
        settings_repo: SettingsRepo,
        mail_fetcher: MailFetcher,
        ai_analyzer: AIAnalyzer,
        reminder_engine: ReminderEngine,
        _skill_loader: SkillLoader,
    ) -> Self {
        AppScheduler {
            item_repo,
            skill_repo,
            account_repo,
            skip_list_repo,
            settings_repo,
            mail_fetcher,
            ai_analyzer,
            reminder_engine,
        }
    }

    fn get_fetch_interval_minutes(&self) -> u64 {
        self.settings_repo
            .get_or("fetchInterval", &DEFAULT_FETCH_INTERVAL.to_string())
            .parse::<u64>()
            .unwrap_or(DEFAULT_FETCH_INTERVAL)
            .max(1)
    }

    pub async fn start_loop(&self, app_handle: AppHandle) {
        let mut reminder_interval = interval(TokioDuration::from_secs(REMINDER_CHECK_INTERVAL_SECS));
        let mut fetch_interval = interval(TokioDuration::from_secs(self.get_fetch_interval_minutes() * 60));
        
        fetch_interval.tick().await;

        loop {
            tokio::select! {
                _ = reminder_interval.tick() => {
                    self.check_reminders(&app_handle).await;
                }
                _ = fetch_interval.tick() => {
                    self.fetch_and_analyze(&app_handle).await;
                    let new_mins = self.get_fetch_interval_minutes();
                    fetch_interval = interval(TokioDuration::from_secs(new_mins * 60));
                    fetch_interval.tick().await;
                }
            }
        }
    }

    async fn check_reminders(&self, app_handle: &AppHandle) {
        let (items_to_notify, _) = self.reminder_engine.check_deadlines();

        if !items_to_notify.is_empty() {
            let mut unique_items = std::collections::HashSet::new();
            let mut contents = Vec::new();

            for item in items_to_notify {
                if unique_items.insert(item.id.clone()) {
                    contents.push(item.content.clone());
                }
            }

            let title = if contents.len() == 1 {
                "即将到期".to_string()
            } else {
                format!("{} 个事项即将到期", contents.len())
            };

            let body = contents.join("\n");

            if let Err(e) = app_handle.notification()
                .builder()
                .title(&title)
                .body(&body)
                .show()
            {
                log::error!("Failed to send notification: {}", e);
            }

            let _ = app_handle.emit("reminder-triggered", serde_json::json!({
                "title": title,
                "body": body,
            }));
        }
    }

    async fn fetch_and_analyze(&self, app_handle: &AppHandle) {
        let _ = app_handle.emit("fetch-status", "正在连接邮箱...");

        let skip_list_repo = self.skip_list_repo.clone();
        let mail_fetcher = self.mail_fetcher.clone();

        let emails = tokio::task::spawn_blocking(move || {
            mail_fetcher.fetch_emails(&skip_list_repo).map_err(|e| e.to_string())
        }).await;

        let emails = match emails {
            Ok(Ok(e)) => e,
            Ok(Err(e)) => {
                log::error!("Failed to fetch emails: {}", e);
                let _ = app_handle.emit("fetch-error", &e);
                return;
            }
            Err(e) => {
                log::error!("Fetch task panicked: {}", e);
                let _ = app_handle.emit("fetch-error", e.to_string());
                return;
            }
        };

        let _ = app_handle.emit("fetch-status", format!("已拉取 {} 封邮件，正在分析...", emails.len()));

        if emails.is_empty() {
            let _ = app_handle.emit("fetch-complete", 0u32);
            return;
        }

        let active_skill = match self.skill_repo.get_active() {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to get active skill: {}", e);
                return;
            }
        };

        let skill_name = active_skill.map(|s| s.name);
        let mut new_items_count = 0;

        let accounts = match self.account_repo.get_all() {
            Ok(a) => a,
            Err(e) => {
                log::error!("Failed to get accounts: {}", e);
                return;
            }
        };

        for (email_idx, email) in emails.iter().enumerate() {
            let _ = app_handle.emit("fetch-status", format!("正在分析第 {}/{} 封...", email_idx + 1, emails.len()));

            let batch_result = self.ai_analyzer.analyze_batch(skill_name.as_deref(), &[email.clone()]).await;

            match batch_result {
                Ok(result) => {
                    for item in result.extracted_items {
                        let matched_skill = skill_name.as_deref();
                        
                        let source_email_id = email.id.as_str();
                        let source_from = email.from.as_str();
                        let source_subject = email.subject.as_str();
                        let source_date = email.date.as_str();
                        let source_account = accounts.iter()
                            .find(|a| a.id == email.account_id)
                            .map(|a| a.email.as_str())
                            .unwrap_or(&email.account_id);

                        match self.item_repo.insert(
                            &item,
                            source_email_id,
                            source_from,
                            source_subject,
                            source_date,
                            source_account,
                            matched_skill,
                        ) {
                            Ok(_) => {
                                new_items_count += 1;
                            }
                            Err(e) => {
                                log::error!("Failed to insert item: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to analyze email {}: {}", email_idx + 1, e);
                }
            }
        }

        if new_items_count > 0 {
            let title = "新事项发现".to_string();
            let body = format!("本次拉取发现了 {} 个新事项", new_items_count);

            if let Err(e) = app_handle.notification()
                .builder()
                .title(title)
                .body(body)
                .show()
            {
                log::error!("Failed to send notification: {}", e);
            }
        }

        let _ = app_handle.emit("fetch-complete", new_items_count as u32);
    }

    pub async fn trigger_fetch_now(&self, app_handle: AppHandle) {
        self.fetch_and_analyze(&app_handle).await;
    }
}
