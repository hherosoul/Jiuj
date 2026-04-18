use crate::models::*;
use crate::db::*;
use crate::services::*;
use tauri::State;
use std::sync::Arc;
use log::info;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AddAIProfileRequest {
    pub name: String,
    pub provider: AIProvider,
    pub model: String,
    #[serde(rename = "baseUrl")]
    pub base_url: Option<String>,
    #[serde(rename = "customName")]
    pub custom_name: Option<String>,
    #[serde(rename = "apiKey")]
    pub api_key: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct UpdateAIProfileRequest {
    pub name: Option<String>,
    pub model: Option<String>,
    #[serde(rename = "baseUrl")]
    pub base_url: Option<Option<String>>,
    #[serde(rename = "customName")]
    pub custom_name: Option<Option<String>>,
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
}

#[tauri::command]
pub async fn get_all_ai_profiles(
    ai_profiles_repo: State<'_, Arc<AIProfilesRepo>>,
) -> Result<Vec<AIProfile>, String> {
    info!("[API] get_all_ai_profiles called");
    match ai_profiles_repo.get_all() {
        Ok(profiles) => {
            info!("[API] get_all_ai_profiles: found {} profiles", profiles.len());
            Ok(profiles)
        }
        Err(e) => {
            info!("[API] get_all_ai_profiles error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_active_ai_profile(
    ai_profiles_repo: State<'_, Arc<AIProfilesRepo>>,
) -> Result<Option<AIProfile>, String> {
    info!("[API] get_active_ai_profile called");
    match ai_profiles_repo.get_active() {
        Ok(profile) => {
            info!("[API] get_active_ai_profile: found = {}", profile.is_some());
            Ok(profile)
        }
        Err(e) => {
            info!("[API] get_active_ai_profile error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn add_ai_profile(
    data: AddAIProfileRequest,
    ai_profiles_repo: State<'_, Arc<AIProfilesRepo>>,
    secret_store: State<'_, Arc<SecretStore>>,
    settings_repo: State<'_, Arc<SettingsRepo>>,
) -> Result<AIProfile, String> {
    info!("[API] add_ai_profile called for name: {}", data.name);
    
    let profile = match ai_profiles_repo.insert(
        &data.name,
        data.provider.clone(),
        &data.model,
        data.base_url.as_deref(),
        data.custom_name.as_deref(),
    ) {
        Ok(p) => p,
        Err(e) => {
            info!("[API] Failed to insert AI profile: {}", e);
            return Err(e.to_string());
        }
    };
    
    info!("[API] AI profile inserted with id: {}", profile.id);

    let key = format!("ai_api_key:{}", profile.id);
    info!("[API] Storing API key with key: {}", key);
    if let Err(e) = secret_store.set_password(&key, &data.api_key, &settings_repo) {
        info!("[API] Failed to store API key: {}", e);
        let _ = ai_profiles_repo.delete(&profile.id);
        return Err(e.to_string());
    }
    info!("[API] API key stored successfully");

    Ok(profile)
}

#[tauri::command]
pub async fn update_ai_profile(
    id: String,
    data: UpdateAIProfileRequest,
    ai_profiles_repo: State<'_, Arc<AIProfilesRepo>>,
    secret_store: State<'_, Arc<SecretStore>>,
    settings_repo: State<'_, Arc<SettingsRepo>>,
) -> Result<(), String> {
    info!("[API] update_ai_profile called for id: {}", id);

    if let Err(e) = ai_profiles_repo.update(
        &id,
        data.name.as_deref(),
        data.model.as_deref(),
        data.base_url,
        data.custom_name,
    ) {
        info!("[API] Failed to update AI profile: {}", e);
        return Err(e.to_string());
    }
    info!("[API] AI profile updated successfully");

    if let Some(api_key) = data.api_key {
        let key = format!("ai_api_key:{}", id);
        info!("[API] Updating API key with key: {}", key);
        if let Err(e) = secret_store.set_password(&key, &api_key, &settings_repo) {
            info!("[API] Failed to update API key: {}", e);
            return Err(e.to_string());
        }
        info!("[API] API key updated successfully");
    }

    Ok(())
}

#[tauri::command]
pub async fn delete_ai_profile(
    id: String,
    ai_profiles_repo: State<'_, Arc<AIProfilesRepo>>,
    secret_store: State<'_, Arc<SecretStore>>,
    settings_repo: State<'_, Arc<SettingsRepo>>,
) -> Result<(), String> {
    info!("[API] delete_ai_profile called for id: {}", id);
    
    let key = format!("ai_api_key:{}", id);
    info!("[API] Deleting API key with key: {}", key);
    let _ = secret_store.delete_password(&key, &settings_repo);
    
    info!("[API] Deleting AI profile from database...");
    match ai_profiles_repo.delete(&id) {
        Ok(_) => {
            info!("[API] AI profile deleted successfully");
            Ok(())
        }
        Err(e) => {
            info!("[API] Failed to delete AI profile: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn set_active_ai_profile(
    id: String,
    ai_profiles_repo: State<'_, Arc<AIProfilesRepo>>,
) -> Result<(), String> {
    info!("[API] set_active_ai_profile called for id: {}", id);
    match ai_profiles_repo.set_active(&id) {
        Ok(_) => {
            info!("[API] AI profile set active successfully");
            Ok(())
        }
        Err(e) => {
            info!("[API] Failed to set active AI profile: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_ai_providers() -> Result<Vec<(String, AIProviderInfo)>, String> {
    info!("[API] get_ai_providers called");
    let providers = vec![
        ("openai".to_string(), AIProviderInfo {
            name: "OpenAI".to_string(),
            models: vec![
                ModelInfo { id: "gpt-4o".to_string(), name: "GPT-4o".to_string() },
                ModelInfo { id: "gpt-4o-mini".to_string(), name: "GPT-4o Mini".to_string() },
                ModelInfo { id: "gpt-4-turbo".to_string(), name: "GPT-4 Turbo".to_string() },
                ModelInfo { id: "gpt-3.5-turbo".to_string(), name: "GPT-3.5 Turbo".to_string() },
            ],
            recommended_model: "gpt-4o".to_string(),
            base_url: Some("https://api.openai.com/v1".to_string()),
        }),
        ("deepseek".to_string(), AIProviderInfo {
            name: "DeepSeek".to_string(),
            models: vec![
                ModelInfo { id: "deepseek-chat".to_string(), name: "DeepSeek Chat".to_string() },
                ModelInfo { id: "deepseek-coder".to_string(), name: "DeepSeek Coder".to_string() },
            ],
            recommended_model: "deepseek-chat".to_string(),
            base_url: Some("https://api.deepseek.com/v1".to_string()),
        }),
        ("kimi".to_string(), AIProviderInfo {
            name: "Kimi".to_string(),
            models: vec![
                ModelInfo { id: "moonshot-v1-8k".to_string(), name: "Moonshot v1 8K".to_string() },
                ModelInfo { id: "moonshot-v1-32k".to_string(), name: "Moonshot v1 32K".to_string() },
                ModelInfo { id: "moonshot-v1-128k".to_string(), name: "Moonshot v1 128K".to_string() },
            ],
            recommended_model: "moonshot-v1-32k".to_string(),
            base_url: Some("https://api.moonshot.cn/v1".to_string()),
        }),
        ("zhipu".to_string(), AIProviderInfo {
            name: "智谱 AI".to_string(),
            models: vec![
                ModelInfo { id: "glm-4".to_string(), name: "GLM-4".to_string() },
                ModelInfo { id: "glm-4-flash".to_string(), name: "GLM-4 Flash".to_string() },
            ],
            recommended_model: "glm-4".to_string(),
            base_url: Some("https://open.bigmodel.cn/api/paas/v4".to_string()),
        }),
        ("qwen".to_string(), AIProviderInfo {
            name: "通义千问".to_string(),
            models: vec![
                ModelInfo { id: "qwen-max".to_string(), name: "Qwen Max".to_string() },
                ModelInfo { id: "qwen-plus".to_string(), name: "Qwen Plus".to_string() },
                ModelInfo { id: "qwen-turbo".to_string(), name: "Qwen Turbo".to_string() },
            ],
            recommended_model: "qwen-plus".to_string(),
            base_url: Some("https://dashscope.aliyuncs.com/compatible-mode/v1".to_string()),
        }),
        ("claude".to_string(), AIProviderInfo {
            name: "Claude".to_string(),
            models: vec![
                ModelInfo { id: "claude-3-5-sonnet-20241022".to_string(), name: "Claude 3.5 Sonnet".to_string() },
                ModelInfo { id: "claude-3-opus-20240229".to_string(), name: "Claude 3 Opus".to_string() },
                ModelInfo { id: "claude-3-sonnet-20240229".to_string(), name: "Claude 3 Sonnet".to_string() },
                ModelInfo { id: "claude-3-haiku-20240307".to_string(), name: "Claude 3 Haiku".to_string() },
            ],
            recommended_model: "claude-3-5-sonnet-20241022".to_string(),
            base_url: Some("https://api.anthropic.com/v1".to_string()),
        }),
        ("ollama".to_string(), AIProviderInfo {
            name: "Ollama".to_string(),
            models: vec![
                ModelInfo { id: "llama3.1".to_string(), name: "Llama 3.1".to_string() },
                ModelInfo { id: "llama3".to_string(), name: "Llama 3".to_string() },
                ModelInfo { id: "gemma2".to_string(), name: "Gemma 2".to_string() },
                ModelInfo { id: "mistral".to_string(), name: "Mistral".to_string() },
            ],
            recommended_model: "llama3.1".to_string(),
            base_url: Some("http://localhost:11434/v1".to_string()),
        }),
        ("custom".to_string(), AIProviderInfo {
            name: "自定义".to_string(),
            models: vec![
                ModelInfo { id: "".to_string(), name: "请输入模型 ID".to_string() },
            ],
            recommended_model: "".to_string(),
            base_url: None,
        }),
    ];
    info!("[API] get_ai_providers returning {} providers", providers.len());
    Ok(providers)
}

#[tauri::command]
pub async fn test_ai_profile(
    data: AddAIProfileRequest,
) -> Result<bool, String> {
    info!("[API] test_ai_profile called for name: {}", data.name);
    
    if data.model.is_empty() {
        return Err("Model is required".to_string());
    }
    
    if data.api_key.is_empty() {
        return Err("API key is required".to_string());
    }

    let provider_str = match data.provider {
        AIProvider::OpenAI => "openai",
        AIProvider::DeepSeek => "deepseek",
        AIProvider::Kimi => "kimi",
        AIProvider::Zhipu => "zhipu",
        AIProvider::Qwen => "qwen",
        AIProvider::Claude => "claude",
        AIProvider::Ollama => "ollama",
        AIProvider::Custom => "custom",
    };

    info!("[API] test_ai_profile: testing connection for provider={}", provider_str);
    
    match AIAnalyzer::test_connection(
        provider_str,
        &data.model,
        &data.api_key,
        data.base_url.as_deref(),
    ).await {
        Ok(_) => {
            info!("[API] test_ai_profile: connection test successful");
            Ok(true)
        }
        Err(e) => {
            info!("[API] test_ai_profile: connection test failed: {}", e);
            Err(e)
        }
    }
}
