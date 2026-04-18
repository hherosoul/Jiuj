use crate::models::*;
use crate::db::*;
use crate::services::*;
use tauri::State;
use std::sync::Arc;
use log::info;

#[tauri::command]
pub async fn get_all_accounts(state: State<'_, Arc<AccountRepo>>) -> Result<Vec<Account>, String> {
    info!("[API] get_all_accounts called");
    match state.get_all() {
        Ok(accounts) => {
            info!("[API] get_all_accounts: found {} accounts", accounts.len());
            Ok(accounts)
        }
        Err(e) => {
            info!("[API] get_all_accounts error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn add_account(
    data: AddAccountRequest,
    account_repo: State<'_, Arc<AccountRepo>>,
    secret_store: State<'_, Arc<SecretStore>>,
    settings_repo: State<'_, Arc<SettingsRepo>>,
) -> Result<Account, String> {
    info!("[API] add_account called for email: {}", data.email);
    
    info!("[API] Testing connection for {}...", data.email);
    if let Err(e) = MailFetcher::new(account_repo.inner().as_ref().clone(), secret_store.inner().as_ref().clone(), Sanitizer::new())
        .test_connection(&data.email, &data.imap_host, data.imap_port, &data.password) {
        info!("[API] Connection test failed: {}", e);
        return Err(e.to_string());
    }
    info!("[API] Connection test successful");

    info!("[API] Inserting account to database...");
    let account = match account_repo.insert(&data.email, &data.imap_host, data.imap_port) {
        Ok(acc) => {
            info!("[API] Account inserted with id: {}", acc.id);
            acc
        }
        Err(e) => {
            info!("[API] Failed to insert account: {}", e);
            return Err(e.to_string());
        }
    };

    let key = format!("password:{}", account.id);
    info!("[API] Storing password with key: {}", key);
    if let Err(e) = secret_store.set_password(&key, &data.password, &settings_repo) {
        info!("[API] Failed to store password: {}", e);
        return Err(e.to_string());
    }
    info!("[API] Password stored successfully");

    Ok(account)
}

#[tauri::command]
pub async fn update_account(
    id: String,
    data: UpdateAccountRequest,
    account_repo: State<'_, Arc<AccountRepo>>,
    secret_store: State<'_, Arc<SecretStore>>,
    settings_repo: State<'_, Arc<SettingsRepo>>,
) -> Result<(), String> {
    info!("[API] update_account called for id: {}", id);
    
    if let Some(email) = &data.email {
        if let Some(password) = &data.password {
            info!("[API] Testing connection for updated account...");
            let host = data.imap_host.as_deref().unwrap_or_default();
            let port = data.imap_port.unwrap_or(993);
            if let Err(e) = MailFetcher::new(account_repo.inner().as_ref().clone(), secret_store.inner().as_ref().clone(), Sanitizer::new())
                .test_connection(email, host, port, password) {
                info!("[API] Connection test failed: {}", e);
                return Err(e.to_string());
            }
            info!("[API] Connection test successful");
        }
    }

    let update = AccountUpdate {
        email: data.email,
        imap_host: data.imap_host,
        imap_port: data.imap_port,
    };
    info!("[API] Updating account in database...");
    if let Err(e) = account_repo.update(&id, update) {
        info!("[API] Failed to update account: {}", e);
        return Err(e.to_string());
    }
    info!("[API] Account updated successfully");

    if let Some(password) = data.password {
        let key = format!("password:{}", id);
        info!("[API] Updating password with key: {}", key);
        if let Err(e) = secret_store.set_password(&key, &password, &settings_repo) {
            info!("[API] Failed to update password: {}", e);
            return Err(e.to_string());
        }
        info!("[API] Password updated successfully");
    }

    Ok(())
}

#[tauri::command]
pub async fn delete_account(
    id: String,
    account_repo: State<'_, Arc<AccountRepo>>,
    secret_store: State<'_, Arc<SecretStore>>,
    settings_repo: State<'_, Arc<SettingsRepo>>,
) -> Result<(), String> {
    info!("[API] delete_account called for id: {}", id);
    
    let key = format!("password:{}", id);
    info!("[API] Deleting password with key: {}", key);
    let _ = secret_store.delete_password(&key, &settings_repo);
    
    info!("[API] Deleting account from database...");
    match account_repo.delete(&id) {
        Ok(_) => {
            info!("[API] Account deleted successfully");
            Ok(())
        }
        Err(e) => {
            info!("[API] Failed to delete account: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn test_account(
    email: String,
    imap_host: String,
    imap_port: u16,
    password: String,
) -> Result<bool, String> {
    info!("[API] test_account called for email: {} at {}:{}", email, imap_host, imap_port);
    
    let tls = native_tls::TlsConnector::builder()
        .build()
        .map_err(|e| format!("TLS error: {}", e))?;
    
    let client = imap::connect((imap_host.as_str(), imap_port), &imap_host, &tls)
        .map_err(|e| format!("Connection failed: {}", e))?;
    
    let mut session = client.login(&email, &password)
        .map_err(|e| format!("Login failed: {}", e.0))?;
    
    session.select("INBOX")
        .map_err(|e| format!("Select INBOX failed: {}", e))?;
    
    session.logout()
        .map_err(|e| format!("Logout failed: {}", e))?;
    
    info!("[API] test_account: Connection successful");
    Ok(true)
}
