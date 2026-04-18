use crate::constants::*;
use crate::models::*;
use crate::db::AIProfilesRepo;
use crate::db::SettingsRepo;
use crate::skills::SkillLoader;
use crate::services::secret_store::SecretStore;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct Message {
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContentBlock>,
}

#[derive(Deserialize)]
struct ClaudeContentBlock {
    text: Option<String>,
}

pub struct BatchResult {
    pub extracted_items: Vec<ExtractedItem>,
}

pub struct AIAnalyzer {
    ai_profiles_repo: AIProfilesRepo,
    settings_repo: SettingsRepo,
    secret_store: SecretStore,
    skill_loader: SkillLoader,
    semaphore: Arc<Semaphore>,
}

impl AIAnalyzer {
    pub fn new(
        ai_profiles_repo: AIProfilesRepo,
        settings_repo: SettingsRepo,
        secret_store: SecretStore,
        skill_loader: SkillLoader,
    ) -> Self {
        AIAnalyzer {
            ai_profiles_repo,
            settings_repo,
            secret_store,
            skill_loader,
            semaphore: Arc::new(Semaphore::new(1)),
        }
    }

    pub fn build_prompt(&self, skill_content: Option<&str>, emails: &[RawEmail]) -> Result<String, Box<dyn std::error::Error>> {
        let mut prompt = String::new();

        prompt.push_str("你是一个邮件信息提取助手。请从以下邮件中提取需要关注的事项。\n\n");

        prompt.push_str("## 提取规则\n");
        prompt.push_str("1. 只提取需要用户关注的事项，忽略普通通知、广告、自动回复、无截止日期且无活动时间的邮件\n");
        prompt.push_str("2. visible 字段：如果该事项需要显示在看板上（有截止日期或活动时间或需要回复），填 true；否则填 false\n");
        prompt.push_str("3. content 字段：用不超过30个字概括事项的核心行动内容，不要包含日期和时间（日期时间已单独提取到 deadline/time 字段），要让人一眼看懂需要做什么\n");
        prompt.push_str("4. deadline 字段：如果有截止日期（如\"请在X日前提交\"），填写截止日期，格式为 ISO 8601（如 2026-04-17T23:59:00+08:00）；没有则填 null\n");
        prompt.push_str("5. time 字段：如果有活动/会议时间（如\"4月17日13:30开会\"），填写活动时间，格式为 ISO 8601（如 2026-04-17T13:30:00+08:00）；没有则填 null\n");
        prompt.push_str("6. priority 字段：根据紧急程度填写 high/medium/low\n");
        prompt.push_str("7. item_type 字段：task（任务）、deadline（截止日期）、reply（需回复）、other（其他）\n\n");

        prompt.push_str("## content 示例（不超过30字，不含日期时间）\n");
        prompt.push_str("- \"提交实验报告\"\n");
        prompt.push_str("- \"参加案例分享会\"\n");
        prompt.push_str("- \"回复合作邀请\"\n");
        prompt.push_str("- \"完成在线AI案例提交\"\n");
        prompt.push_str("- \"确认参会并填写回执\"\n\n");

        prompt.push_str("## visible 判断规则\n");
        prompt.push_str("- 有截止日期（deadline 不为 null）→ visible: true\n");
        prompt.push_str("- 有活动时间（time 不为 null）→ visible: true\n");
        prompt.push_str("- 需要回复（item_type 为 reply）→ visible: true\n");
        prompt.push_str("- 普通通知、广告、无明确行动项 → visible: false\n\n");

        if let Some(skill) = skill_content {
            prompt.push_str("## 自定义规则（优先于上述默认规则）\n");
            prompt.push_str(skill);
            prompt.push_str("\n\n");
        }

        prompt.push_str("## 输出格式\n");
        prompt.push_str("必须严格按以下 JSON 格式输出，不要输出其他内容：\n");
        prompt.push_str(r#"{"items":[{"content":"不超过30字的核心行动总结","deadline":"2026-04-17T23:59:00+08:00","time":"2026-04-17T13:30:00+08:00","visible":true,"priority":"high","item_type":"task"}]}"#);
        prompt.push_str("\n\n");
        prompt.push_str("注意：\n");
        prompt.push_str("- deadline 和 time 没有则填 null，不要填占位符如 {date}\n");
        prompt.push_str("- deadline 是截止日期，time 是活动/会议时间，两者是不同的概念\n");
        prompt.push_str("- visible 由你根据上述规则判断，不是所有邮件都需要显示\n");
        prompt.push_str("- content 不要包含日期时间，只写核心行动内容，让人一眼看懂要做什么\n\n");

        prompt.push_str("## 待分析邮件\n");
        for email in emails {
            prompt.push_str("---\n");
            prompt.push_str(&format!("发件人: {}\n", email.from));
            prompt.push_str(&format!("主题: {}\n", email.subject));
            prompt.push_str(&format!("日期: {}\n", email.date));
            if email.truncated {
                prompt.push_str("[内容已截断]\n");
            }
            prompt.push_str(&format!("正文:\n{}\n", email.body));
        }

        Ok(prompt)
    }

    pub async fn analyze_batch(&self, skill_name: Option<&str>, emails: &[RawEmail]) -> Result<BatchResult, Box<dyn std::error::Error>> {
        let _permit = self.semaphore.acquire().await?;

        let profile = match self.ai_profiles_repo.get_active()? {
            Some(p) => p,
            None => {
                log::warn!("No active AI profile found, skipping analysis");
                return Ok(BatchResult { extracted_items: vec![] });
            }
        };

        let provider_str = match profile.provider {
            AIProvider::OpenAI => "openai",
            AIProvider::DeepSeek => "deepseek",
            AIProvider::Kimi => "kimi",
            AIProvider::Zhipu => "zhipu",
            AIProvider::Qwen => "qwen",
            AIProvider::Claude => "claude",
            AIProvider::Ollama => "ollama",
            AIProvider::Custom => "custom",
        };

        let api_key_key = format!("ai_api_key:{}", profile.id);
        let api_key = match self.secret_store.get_password(&api_key_key, &self.settings_repo)? {
            Some(k) => k,
            None => {
                log::warn!("No API key found for profile {}, skipping analysis", profile.name);
                return Ok(BatchResult { extracted_items: vec![] });
            }
        };

        let skill_content = skill_name.and_then(|n| self.skill_loader.load_skill_content(n));
        let prompt = self.build_prompt(skill_content.as_deref(), emails)?;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(LLM_TIMEOUT_SECS))
            .build()?;

        let is_claude = provider_str == "claude";

        if is_claude {
            self.call_claude_api(&client, &profile.model, &api_key, &prompt).await
        } else {
            let url = profile.base_url.as_deref()
                .map(|s| {
                    if s.ends_with("/chat/completions") {
                        s.to_string()
                    } else if s.ends_with('/') {
                        format!("{}chat/completions", s)
                    } else {
                        format!("{}/chat/completions", s)
                    }
                })
                .unwrap_or_else(|| self.get_default_base_url(provider_str));

            self.call_openai_compatible_api(&client, &url, &profile.model, &api_key, &prompt).await
        }
    }

    async fn call_openai_compatible_api(
        &self,
        client: &Client,
        url: &str,
        model: &str,
        api_key: &str,
        prompt: &str,
    ) -> Result<BatchResult, Box<dyn std::error::Error>> {
        let request_body = serde_json::json!({
            "model": model,
            "messages": [
                {"role": "system", "content": "你是一个邮件信息提取助手，从邮件中提取需要关注的事项，以 JSON 格式输出。"},
                {"role": "user", "content": prompt}
            ],
            "response_format": {"type": "json_object"},
            "temperature": 0.1,
        });

        let response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            log::error!("AI API error: status={}, body={}", status, body);
            return Ok(BatchResult { extracted_items: vec![] });
        }

        let body_text = response.text().await?;
        let log_len = std::cmp::min(body_text.chars().count(), 300);
        let log_content: String = body_text.chars().take(log_len).collect();
        log::debug!("AI API response (truncated): {}", log_content);
        
        let chat_response: ChatCompletionResponse = match serde_json::from_str(&body_text) {
            Ok(r) => r,
            Err(e) => {
                log::error!("Failed to deserialize AI API response: {}. Body: {}", e, body_text);
                return Ok(BatchResult { extracted_items: vec![] });
            }
        };
        
        let content = match chat_response.choices.first() {
            Some(c) if !c.message.content.is_empty() => c.message.content.clone(),
            _ => {
                log::error!("Empty response from AI API");
                return Ok(BatchResult { extracted_items: vec![] });
            }
        };

        self.parse_llm_response(&content)
    }

    async fn call_claude_api(
        &self,
        client: &Client,
        model: &str,
        api_key: &str,
        prompt: &str,
    ) -> Result<BatchResult, Box<dyn std::error::Error>> {
        let url = "https://api.anthropic.com/v1/messages";

        let request_body = serde_json::json!({
            "model": model,
            "max_tokens": 4096,
            "system": "你是一个邮件信息提取助手，从邮件中提取需要关注的事项，以 JSON 格式输出。",
            "messages": [
                {"role": "user", "content": prompt}
            ],
        });

        let response = client
            .post(url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            log::error!("Claude API error: status={}, body={}", status, body);
            return Ok(BatchResult { extracted_items: vec![] });
        }

        let body_text = response.text().await?;
        log::debug!("Claude API response: {}", &body_text[..body_text.len().min(1000)]);
        
        let claude_response: ClaudeResponse = match serde_json::from_str(&body_text) {
            Ok(r) => r,
            Err(e) => {
                log::error!("Failed to deserialize Claude API response: {}. Body: {}", e, body_text);
                return Ok(BatchResult { extracted_items: vec![] });
            }
        };
        
        let content = claude_response.content
            .iter()
            .filter_map(|block| block.text.clone())
            .collect::<Vec<_>>()
            .join("");

        if content.is_empty() {
            log::error!("Empty response from Claude API");
            return Ok(BatchResult { extracted_items: vec![] });
        }

        self.parse_llm_response(&content)
    }

    fn parse_llm_response(&self, content: &str) -> Result<BatchResult, Box<dyn std::error::Error>> {
        let json_str = if content.trim().starts_with('{') || content.trim().starts_with('[') {
            content.to_string()
        } else {
            let start = content.find('{').unwrap_or(0);
            let end = content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
            content[start..end].to_string()
        };

        let llm_response: LLMResponse = match serde_json::from_str(&json_str) {
            Ok(r) => r,
            Err(e) => {
                log::error!("Failed to parse AI response as JSON: {}. Raw content: {}", e, &content[..content.len().min(500)]);
                return Ok(BatchResult { extracted_items: vec![] });
            }
        };

        let valid_items = llm_response.items.into_iter().filter_map(|mut item| {
            log::info!("[AIAnalyzer] Processing item: content={:?}, deadline={:?}, time={:?}, visible={:?}", item.content, item.deadline, item.time, item.visible);

            if !item.visible.unwrap_or(false) {
                log::info!("[AIAnalyzer] Item visible=false, skipping");
                return None;
            }

            if item.content.trim().is_empty() {
                log::warn!("[AIAnalyzer] Empty content, skipping item");
                return None;
            }

            let content_chars = item.content.chars().count();
            if content_chars > 20 {
                let truncated: String = item.content.chars().take(17).collect();
                item.content = format!("{}...", truncated);
                log::info!("[AIAnalyzer] Content truncated to 20 chars: {:?}", item.content);
            }

            if let Some(deadline) = &item.deadline {
                if deadline == "{date}" || deadline.trim().is_empty() {
                    log::warn!("[AIAnalyzer] Found placeholder/empty deadline, setting to None");
                    item.deadline = None;
                } else if chrono::DateTime::parse_from_rfc3339(deadline).is_err() {
                    log::warn!("[AIAnalyzer] Invalid deadline format: {}, setting to None", deadline);
                    item.deadline = None;
                }
            }

            if let Some(time) = &item.time {
                if time == "{date}" || time.trim().is_empty() {
                    log::warn!("[AIAnalyzer] Found placeholder/empty time, setting to None");
                    item.time = None;
                } else if chrono::DateTime::parse_from_rfc3339(time).is_err() {
                    log::warn!("[AIAnalyzer] Invalid time format: {}, setting to None", time);
                    item.time = None;
                }
            }

            Some(item)
        }).collect::<Vec<_>>();

        Ok(BatchResult { extracted_items: valid_items })
    }

    pub async fn test_connection(
        provider: &str,
        model: &str,
        api_key: &str,
        base_url: Option<&str>,
    ) -> Result<bool, String> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let is_claude = provider == "claude";

        if is_claude {
            let request_body = serde_json::json!({
                "model": model,
                "max_tokens": 10,
                "messages": [
                    {"role": "user", "content": "Hi"}
                ],
            });

            let response = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await
                .map_err(|e| format!("Connection error: {}", e))?;

            if response.status().is_success() {
                Ok(true)
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                Err(format!("API returned status {}: {}", status, body))
            }
        } else {
            let url = base_url
                .map(|s| {
                    if s.ends_with("/chat/completions") {
                        s.to_string()
                    } else if s.ends_with('/') {
                        format!("{}chat/completions", s)
                    } else {
                        format!("{}/chat/completions", s)
                    }
                })
                .unwrap_or_else(|| {
                    let defaults = get_default_base_urls();
                    defaults.get(provider).cloned().unwrap_or_else(|| "https://api.openai.com/v1/chat/completions".to_string())
                });

            let request_body = serde_json::json!({
                "model": model,
                "messages": [
                    {"role": "user", "content": "Hi"}
                ],
                "max_tokens": 5,
            });

            let response = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await
                .map_err(|e| format!("Connection error: {}", e))?;

            if response.status().is_success() {
                Ok(true)
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                Err(format!("API returned status {}: {}", status, body))
            }
        }
    }

    fn get_default_base_url(&self, provider: &str) -> String {
        let defaults = get_default_base_urls();
        defaults.get(provider).cloned().unwrap_or_else(|| "https://api.openai.com/v1/chat/completions".to_string())
    }
}

fn get_default_base_urls() -> std::collections::HashMap<&'static str, String> {
    let mut map = std::collections::HashMap::new();
    map.insert("openai", "https://api.openai.com/v1/chat/completions".to_string());
    map.insert("deepseek", "https://api.deepseek.com/v1/chat/completions".to_string());
    map.insert("kimi", "https://api.moonshot.cn/v1/chat/completions".to_string());
    map.insert("zhipu", "https://open.bigmodel.cn/api/paas/v4/chat/completions".to_string());
    map.insert("qwen", "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions".to_string());
    map.insert("ollama", "http://localhost:11434/v1/chat/completions".to_string());
    map
}
