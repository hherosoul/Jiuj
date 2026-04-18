use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    Task,
    Deadline,
    Reply,
    Notification,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ItemStatus {
    Pending,
    Overdue,
}

impl ItemStatus {
    pub fn to_str(&self) -> &'static str {
        match self {
            ItemStatus::Pending => "pending",
            ItemStatus::Overdue => "overdue",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "overdue" => ItemStatus::Overdue,
            _ => ItemStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    Active,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SkipType {
    Sender,
    Domain,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AIProvider {
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "deepseek")]
    DeepSeek,
    #[serde(rename = "kimi")]
    Kimi,
    #[serde(rename = "zhipu")]
    Zhipu,
    #[serde(rename = "qwen")]
    Qwen,
    #[serde(rename = "claude")]
    Claude,
    #[serde(rename = "ollama")]
    Ollama,
    #[serde(rename = "custom")]
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub content: String,
    pub deadline: Option<String>,
    pub time: Option<String>,
    pub priority: Priority,
    #[serde(rename = "itemType")]
    pub item_type: ItemType,
    #[serde(rename = "remindOffsets")]
    pub remind_offsets: Option<Vec<u64>>,
    #[serde(rename = "notifiedStages")]
    pub notified_stages: Vec<usize>,
    #[serde(rename = "sourceEmailId")]
    pub source_email_id: String,
    #[serde(rename = "sourceFrom")]
    pub source_from: String,
    #[serde(rename = "sourceSubject")]
    pub source_subject: String,
    #[serde(rename = "sourceDate")]
    pub source_date: String,
    #[serde(rename = "sourceAccount")]
    pub source_account: String,
    #[serde(rename = "matchedSkill")]
    pub matched_skill: Option<String>,
    pub status: ItemStatus,
    #[serde(rename = "lastNotifiedAt")]
    pub last_notified_at: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "completedAt")]
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub email: String,
    #[serde(rename = "imapHost")]
    pub imap_host: String,
    #[serde(rename = "imapPort")]
    pub imap_port: u16,
    #[serde(rename = "lastUid")]
    pub last_uid: u64,
    pub status: AccountStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    #[serde(rename = "sortOrder")]
    pub sort_order: i32,
    #[serde(rename = "isBuiltin")]
    pub is_builtin: bool,
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub skip_type: SkipType,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedItem {
    pub content: String,
    pub deadline: Option<String>,
    pub time: Option<String>,
    pub visible: Option<bool>,
    pub priority: Priority,
    pub item_type: ItemType,
    pub remind_offsets: Option<Vec<u64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub items: Vec<ExtractedItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProviderInfo {
    pub name: String,
    pub models: Vec<ModelInfo>,
    #[serde(rename = "recommendedModel")]
    pub recommended_model: String,
    #[serde(rename = "baseUrl")]
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProfile {
    pub id: String,
    pub name: String,
    pub provider: AIProvider,
    pub model: String,
    #[serde(rename = "baseUrl")]
    pub base_url: Option<String>,
    #[serde(rename = "customName")]
    pub custom_name: Option<String>,
    pub is_active: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawEmail {
    pub id: String,
    pub from: String,
    pub subject: String,
    pub date: String,
    pub body: String,
    pub account_id: String,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTemplate {
    pub name: String,
    pub description: String,
    pub sections: SkillTemplateSections,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTemplateSections {
    pub identity: String,
    #[serde(rename = "extractRules")]
    pub extract_rules: String,
    #[serde(rename = "priorityRules")]
    pub priority_rules: String,
    #[serde(rename = "notifyRules")]
    pub notify_rules: String,
    #[serde(rename = "customPrompt")]
    pub custom_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAccountRequest {
    pub email: String,
    #[serde(rename = "imapHost")]
    pub imap_host: String,
    #[serde(rename = "imapPort")]
    pub imap_port: u16,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccountRequest {
    pub email: Option<String>,
    #[serde(rename = "imapHost")]
    pub imap_host: Option<String>,
    #[serde(rename = "imapPort")]
    pub imap_port: Option<u16>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSkill {
    pub name: String,
    pub description: String,
    #[serde(rename = "sortOrder")]
    pub sort_order: i32,
    #[serde(rename = "isBuiltin")]
    pub is_builtin: bool,
    #[serde(rename = "filePath")]
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSkipEntry {
    #[serde(rename = "type")]
    pub skip_type: SkipType,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUpdate {
    pub email: Option<String>,
    pub imap_host: Option<String>,
    pub imap_port: Option<u16>,
}
