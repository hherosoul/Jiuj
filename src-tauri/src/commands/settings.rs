use crate::models::*;
use crate::db::*;
use crate::services::*;
use tauri::State;
use std::sync::Arc;
use log::info;

#[tauri::command]
pub async fn get_settings(settings_repo: State<'_, Arc<SettingsRepo>>) -> Result<Vec<Setting>, String> {
    info!("[API] get_settings called");
    match settings_repo.get_all(None::<Vec<String>>) {
        Ok(map) => {
            let settings: Vec<Setting> = map.into_iter().map(|(key, value)| Setting { key, value }).collect();
            info!("[API] get_settings: found {} settings", settings.len());
            Ok(settings)
        }
        Err(e) => {
            info!("[API] get_settings error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn set_settings(
    key: String,
    value: String,
    settings_repo: State<'_, Arc<SettingsRepo>>,
) -> Result<(), String> {
    info!("[API] set_settings called: key = {}", key);
    match settings_repo.set(&key, &value) {
        Ok(_) => {
            info!("[API] set_settings: success");
            Ok(())
        }
        Err(e) => {
            info!("[API] set_settings error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_skip_list(state: State<'_, Arc<SkipListRepo>>) -> Result<Vec<SkipEntry>, String> {
    info!("[API] get_skip_list called");
    match state.get_all() {
        Ok(entries) => {
            info!("[API] get_skip_list: found {} entries", entries.len());
            Ok(entries)
        }
        Err(e) => {
            info!("[API] get_skip_list error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn add_skip_entry(
    entry: NewSkipEntry,
    state: State<'_, Arc<SkipListRepo>>,
) -> Result<SkipEntry, String> {
    info!("[API] add_skip_entry called: type = {:?}, value = {}", entry.skip_type, entry.value);
    match state.insert(entry) {
        Ok(entry) => {
            info!("[API] add_skip_entry: success, id = {}", entry.id);
            Ok(entry)
        }
        Err(e) => {
            info!("[API] add_skip_entry error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn delete_skip_entry(
    id: String,
    state: State<'_, Arc<SkipListRepo>>,
) -> Result<(), String> {
    info!("[API] delete_skip_entry called for id: {}", id);
    match state.delete(&id) {
        Ok(_) => {
            info!("[API] delete_skip_entry: success");
            Ok(())
        }
        Err(e) => {
            info!("[API] delete_skip_entry error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_sanitize_rules() -> Vec<(String, String, String)> {
    info!("[API] get_sanitize_rules called");
    let sanitizer = Sanitizer::new();
    let rules: Vec<(String, String, String)> = sanitizer.get_rules().into_iter().map(|(n, p, r)| (n.to_string(), p.to_string(), r.to_string())).collect();
    info!("[API] get_sanitize_rules: found {} rules", rules.len());
    rules
}

#[tauri::command]
pub async fn get_skill_template() -> SkillTemplate {
    info!("[API] get_skill_template called");
    SkillTemplate {
        name: "".to_string(),
        description: "".to_string(),
        sections: SkillTemplateSections {
            identity: "# identity\n".to_string(),
            extract_rules: "## extract-rules\n".to_string(),
            priority_rules: "## priority-rules\n".to_string(),
            notify_rules: "## notify-rules\n".to_string(),
            custom_prompt: "## custom-prompt\n".to_string(),
        },
    }
}

#[tauri::command]
pub async fn test_email_body_sanitization(
    body: String,
) -> Result<(String, bool), String> {
    info!("[API] test_email_body_sanitization called, body length: {}", body.len());
    let sanitizer = Sanitizer::new();
    let result = sanitizer.process(&body);
    info!("[API] test_email_body_sanitization: processed, modified = {}", result.1);
    Ok(result)
}
