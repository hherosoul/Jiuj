# Jiuj — 架构设计文档

> 基于 Jiuj-PRD.md，采用 Tauri v2 + Rust 后端 + React 前端，按 MVC 模式拆分为 Model / View / Controller 三层。

---

## 0. 技术栈版本约束

| 组件 | 版本要求 | 说明 |
|------|---------|------|
| Tauri | >= 2.0 | Rust 后端 + WebView 前端，无 Node.js 原生模块 ABI 问题 |
| Rust | >= 1.77 | 2024 edition，严格类型安全 |
| TypeScript | >= 5.0 | 严格模式（`strict: true`） |
| React | >= 18 | 函数组件 + Hooks |
| Tailwind CSS | >= 3.x | JIT 模式 |
| Radix UI | latest | 无样式 UI 组件（对话框、下拉菜单等） |
| react-router-dom | >= 6 | Hash Router（桌面应用无需路径） |
| TanStack Query | >= 5 | Tauri command 缓存 + 后台推送自动刷新 |
| react-hook-form | >= 7 | 表单管理 |
| i18next | >= 23 | 国际化 |
| Vite | >= 5 | 前端构建（Tauri 内置集成） |
| rusqlite | >= 0.31 | Rust 原生 SQLite 绑定，静态编译进二进制 |
| imap | >= 2.4 | Rust IMAP 客户端 |
| mail-parser | >= 0.9 | Rust 邮件解析 |
| reqwest | >= 0.12 | Rust HTTP 客户端（LLM API 调用） |
| regex | >= 1.10 | Rust 正则引擎（脱敏处理） |
| aes-gcm | >= 0.10 | AES-256-GCM 对称加密（凭证加密存储到 SQLite） |
| rand | >= 0.8 | 加密密钥随机生成 |
| serde / serde_json | latest | Rust 序列化/反序列化 |
| log + env_logger | latest | Rust 日志记录 |
| uuid | latest | UUID 生成 |

---

## 1. 功能清单总览

| # | 功能域 | 功能点 | PRD 章节 |
|---|--------|--------|----------|
| F1 | 邮箱管理 | 添加/编辑/删除邮箱、连接测试、拉取频率配置、多账号 | 4.1 |
| F2 | 邮件拉取 | 定时拉取新邮件、跳过名单过滤、UID 断点续拉、本地预处理（脱敏+格式化+截断）、手动全量重扫（rescan） | 4.1 + 4.7 |
| F3 | 数据脱敏 | 正则替换手机号/身份证/银行卡/邮箱/IP 等 | 4.6 |
| F4 | AI 分析 | 分批批量调用 LLM（每批 10 封）、Skill Prompt 拼接、解析 JSON 结果 | 4.2 + 4.5 |
| F5 | 事项管理 | 存储、查询、标记完成、忽略、来源追溯 | 4.3 |
| F6 | 看板展示 | 待办列表（按时间/优先级排列）、已折叠、事项详情 | 4.3 + 5.2 |
| F7 | 提醒通知 | 新事项弹窗、两段式到期提醒（提前 1 天 + 2 小时）、过期边界处理、免打扰时段、原生通知 | 4.4 |
| F8 | Skills 管理 | 浏览/新建/编辑/导入/导出 Skill、单选切换、内置默认 Skill | 4.5 + 5.5 |
| F9 | Skill 编辑器 | 模板填空式编辑、保存/重置 | 5.6 |
| F10 | 跳过名单 | 发件人/域名跳过、看板快捷添加、确认弹窗 | 4.7 |
| F11 | AI 设置 | 厂商选择、API Key 管理、测试连接 | 4.8 |
| F12 | 通用设置 | 语言切换、开机启动、关闭行为、默认提醒时间（两段式）、免打扰时段 | 5.8 |
| F13 | 系统托盘 | 最小化到托盘、右键菜单、状态图标 | 4.9 |
| F14 | i18n | 跟随系统/手动切换、所有 UI 文本外置 | 5.1 |

---

## 2. MVC 分层总览

```
┌─────────────────────────────────────────────────────────────┐
│                        View（WebView 前端）                   │
│  Pages: Board / Accounts / AISettings / Skills / SkillEditor │
│         SkipList / General                                   │
│  Components: ItemCard / ItemDetailDialog / ConfirmDialog /           │
│              SkillTemplateEditor / BoardEmptyState                  │
│              SkillTemplateEditor                             │
│  Hooks: useTauriCommand                                     │
│  Lib: tauri-api (类型安全封装)                                │
├──────────────────────────────────────────────────────────────┤
│             Tauri IPC（invoke / listen 通道）                 │
├──────────────────────────────────────────────────────────────┤
│                Controller（Tauri Commands）                   │
│  src-tauri/src/commands/                                     │
│  item_commands / account_commands / skill_commands /          │
│  skip_commands / settings_commands / ai                       │
├──────────────────────────────────────────────────────────────┤
│                        Model（Rust 后端）                     │
│  Services: src-tauri/src/services/                           │
│            MailFetcher / AIAnalyzer / ReminderEngine /       │
│            Sanitizer / AppScheduler / TrayManager /          │
│            SecretStore                                       │
│  Repos:    src-tauri/src/db/                                 │
│            ItemRepo / AccountRepo / AIProfilesRepo /         │
│            SkillRepo / SkipListRepo / SettingsRepo            │
│  Core:     src-tauri/src/db/Database /                       │
│            src-tauri/src/skills/SkillLoader                   │
│  Models:   src-tauri/src/models.rs                           │
└─────────────────────────────────────────────────────────────┘
```

**与 Electron 版的关键差异：**
- 无 preload 层——Tauri 的 `invoke` 机制本身就是安全桥接，无需手动 contextBridge
- 无 IPC 通道字符串——Tauri command 是 Rust 函数，编译期类型检查，不存在拼写错误
- 无原生模块 ABI 问题——所有 Rust crate 静态编译进二进制

---

## 3. Shared — 数据模型

> 位于 `src-tauri/src/models.rs`，Rust 后端使用。
> 前端通过 `src/lib/tauri-api.ts` 定义对应的 TypeScript 类型，保持同步。

### 3.1 Rust 类型定义 `models.rs`

```rust
use serde::{Deserialize, Serialize};

// ──────────────────────────────────────
// 枚举类型

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
    Completed,
    Ignored,
}

impl ItemStatus {
    pub fn to_str(&self) -> &'static str {
        match self {
            ItemStatus::Pending => "pending",
            ItemStatus::Completed => "completed",
            ItemStatus::Ignored => "ignored",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "completed" => ItemStatus::Completed,
            "ignored" => ItemStatus::Ignored,
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
pub enum TrayStatus {
    Normal,
    Paused,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum AIProvider {
    OpenAI,
    DeepSeek,
    Kimi,
    Zhipu,
    Qwen,
    Claude,
    Ollama,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Locale {
    #[serde(rename = "zh-CN")]
    ZhCN,
    #[serde(rename = "en-US")]
    EnUS,
    #[serde(rename = "auto")]
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CloseAction {
    MinimizeToTray,
    Quit,
}

// ──────────────────────────────────────
// 数据实体

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,                          // UUID
    pub content: String,
    pub deadline: Option<String>,            // ISO 8601
    pub time: Option<String>,                // ISO 8601，活动/会议时间
    pub priority: Priority,
    #[serde(rename = "itemType")]
    pub item_type: ItemType,
    /// 提醒提前量（分钟数组）。
    /// - None = 使用用户默认值（通用设置页配置）
    /// - Some(vec![]) = 不触发到期提醒（仅新事项提取时通知一次）
    /// - Some(vec![1440, 120]) = 提前 1 天 + 提前 2 小时
    #[serde(rename = "remindOffsets")]
    pub remind_offsets: Option<Vec<u64>>,
    /// 已通知的阶段索引，如 vec![0, 1] 表示两段都已提醒
    #[serde(rename = "notifiedStages")]
    pub notified_stages: Vec<usize>,
    #[serde(rename = "sourceEmailId")]
    pub source_email_id: String,
    #[serde(rename = "sourceFrom")]
    pub source_from: String,
    #[serde(rename = "sourceSubject")]
    pub source_subject: String,
    #[serde(rename = "sourceAccount")]
    pub source_account: String,
    #[serde(rename = "sourceDate")]
    pub source_date: String,                // ISO 8601，发件日期
    #[serde(rename = "matchedSkill")]
    pub matched_skill: Option<String>,
    pub status: ItemStatus,
    #[serde(rename = "lastNotifiedAt")]
    pub last_notified_at: Option<String>,    // ISO 8601
    #[serde(rename = "createdAt")]
    pub created_at: String,                  // ISO 8601
    #[serde(rename = "completedAt")]
    pub completed_at: Option<String>,        // ISO 8601
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
    pub updated_at: String,                  // ISO 8601
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

// ──────────────────────────────────────
// 请求/响应结构

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedItem {
    pub content: String,
    pub deadline: Option<String>,
    pub time: Option<String>,                // 活动/会议时间
    pub visible: Option<bool>,               // 是否显示在看板上
    pub priority: Priority,
    pub item_type: ItemType,
    /// 提醒提前量（分钟数组），None = 使用用户默认值，Some([]) = 不提醒
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
    pub created_at: String,                  // ISO 8601
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawEmail {
    pub id: String,
    pub from: String,
    pub subject: String,
    pub date: String,
    pub body: String,
    pub account_id: String,
    /// 邮件正文是否被截断（超过单封上限时为 true）
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

/// 前端添加邮箱的请求体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAccountRequest {
    pub email: String,
    #[serde(rename = "imapHost")]
    pub imap_host: String,
    #[serde(rename = "imapPort")]
    pub imap_port: u16,
    pub password: String,
}

/// 前端更新邮箱的请求体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccountRequest {
    pub email: Option<String>,
    #[serde(rename = "imapHost")]
    pub imap_host: Option<String>,
    #[serde(rename = "imapPort")]
    pub imap_port: Option<u16>,
    pub password: Option<String>,            // 有值时更新密码
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
    #[serde(rename = "imapHost")]
    pub imap_host: Option<String>,
    #[serde(rename = "imapPort")]
    pub imap_port: Option<u16>,
}
```

### 3.2 常量定义 `src-tauri/src/constants.rs`

```rust
pub const APP_NAME: &str = "Jiuj";
pub const DB_NAME: &str = "jiuj.db";
pub const BUILTIN_SKILL_NAME: &str = "default";
pub const DEFAULT_FETCH_INTERVAL: u64 = 15;          // 分钟
pub const DEFAULT_REMIND_OFFSETS: [u64; 2] = [1440, 120]; // 默认提醒：提前 1 天 + 2 小时（分钟）
pub const REMINDER_CHECK_INTERVAL_SECS: u64 = 60;     // 提醒引擎扫描间隔：1 分钟
pub const LLM_TIMEOUT_SECS: u64 = 30;                 // LLM API 超时：30 秒
pub const MAX_EMAIL_BODY_LENGTH: usize = 8000;        // 单封邮件 body 最大 UTF-8 字符数（直接尾部截断）
```

> **注意：** AI 厂商配置（`AIProviderInfo`/`ModelInfo`）已改为运行时动态构建的结构体（`String` 类型），定义在 `models.rs` 中，在 `commands/ai.rs` 的 `get_ai_providers` 命令中返回给前端。不再使用 `&'static str` 常量形式。

---

## 4. Model 层 — Rust 后端

> 位于 `src-tauri/src/`，负责业务逻辑和数据访问。

### 4.0 应用启动流程

> `src-tauri/src/main.rs` 负责创建所有依赖实例并按顺序初始化。

```rust
// src-tauri/src/main.rs — 伪代码，展示启动顺序

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            // 1. 数据库
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let db_path = format!("{}/{}", app_data_dir.display(), DB_NAME);
            let db = Database::new(&db_path)?;
            db.init()?;

            // 2. 安全模块（AES-256-GCM 加密，密钥存入 ~/.jiuj/.key）
            let secret_store = SecretStore::new(&app_data_dir);

            // 3. Repository 层
            let item_repo = ItemRepo::new(db.clone());
            let account_repo = AccountRepo::new(db.clone());
            let ai_profiles_repo = AIProfilesRepo::new(db.clone());
            let skill_repo = SkillRepo::new(db.clone());
            let skip_list_repo = SkipListRepo::new(db.clone());
            let settings_repo = SettingsRepo::new(db);

            // 4. Skill 文件加载器
            let skill_dir = shellexpand::tilde(SKILL_DIR).to_string();
            let skill_loader = SkillLoader::new(&skill_dir);
            skill_loader.ensure_builtin_skill("src-tauri/skills/builtin/default")?;

            // 5. Service 层
            let sanitizer = Sanitizer::new();
            let ai_analyzer = AIAnalyzer::new(
                ai_profiles_repo.clone(),
                settings_repo.clone(),
                secret_store.clone(),
                skill_loader.clone(),
            );
            let reminder_engine = ReminderEngine::new(
                item_repo.clone(),
                settings_repo.clone(),
            );
            let mail_fetcher = MailFetcher::new(
                account_repo.clone(),
                skip_list_repo.clone(),
                secret_store.clone(),
            );
            let app_scheduler = AppScheduler::new(
                mail_fetcher,
                ai_analyzer,
                reminder_engine.clone(),
                item_repo.clone(),
            );

            // 6. 系统级行为配置
            // 开机启动
            let auto_start = settings_repo.get("autoStart")
                .unwrap_or_else(|| "false".to_string());
            // 通过 tauri-plugin-autostart 管理

            // 关闭窗口行为：由前端 + Tauri 事件控制
            // 在 tauri.conf.json 中配置 "visibleOnAllWorkspaces": false

            // 7. 启动后台服务
            let interval = settings_repo.get("fetchInterval")
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(DEFAULT_FETCH_INTERVAL);
            app_scheduler.start_fetch(interval);
            reminder_engine.start(REMINDER_CHECK_INTERVAL_SECS);

            // 8. 管理 Tauri 状态（所有 Repo + Service 注入）
            app.manage(item_repo);
            app.manage(account_repo);
            app.manage(ai_profiles_repo);
            app.manage(skill_repo);
            app.manage(skip_list_repo);
            app.manage(settings_repo);
            app.manage(secret_store);
            app.manage(skill_loader);
            app.manage(ai_analyzer);
            app.manage(reminder_engine);
            app.manage(app_scheduler);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 事项
            item_commands::get_all_items,
            item_commands::get_pending_items,
            item_commands::get_completed_items,
            item_commands::complete_item,
            item_commands::ignore_item,
            item_commands::restore_item,
            item_commands::get_item_detail,
            // 账号
            account_commands::get_all_accounts,
            account_commands::add_account,
            account_commands::update_account,
            account_commands::delete_account,
            account_commands::test_account_connection,
    // 拉取
    item_commands::start_fetch,
    item_commands::stop_fetch,
    item_commands::get_fetch_status,
    item_commands::set_fetch_interval,
    item_commands::rescan_recent,
            // AI
            ai::get_all_ai_profiles,
            ai::get_active_ai_profile,
            ai::add_ai_profile,
            ai::update_ai_profile,
            ai::delete_ai_profile,
            ai::set_active_ai_profile,
            ai::get_ai_providers,
            ai::test_ai_profile,
            // Skill
            skill_commands::get_all_skills,
            skill_commands::get_active_skill,
            skill_commands::set_active_skill,
            skill_commands::deactivate_skill,
            skill_commands::get_skill_content,
            skill_commands::save_skill_content,
            skill_commands::create_skill,
            skill_commands::delete_skill,
            skill_commands::update_skill_sort_order,
            skill_commands::reset_skill,
            skill_commands::export_skill,
            skill_commands::import_skill,
            // 跳过名单
            skip_commands::get_all_skip_entries,
            skip_commands::add_skip_entry,
            skip_commands::delete_skip_entry,
            skip_commands::add_skip_from_item,
            // 通用设置
            settings_commands::get_setting,
            settings_commands::set_setting,
            settings_commands::get_all_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Jiuj");
}
```

**Tauri 安全配置（tauri.conf.json 关键项）：**

```json
{
  "app": {
    "windows": [
      {
        "title": "Jiuj",
        "visible": false,
        "decorations": true
      }
    ],
    "security": {
      "csp": "default-src 'self'"
    }
  },
  "plugins": {
    "notification": {
      "all": true
    },
    "autostart": {
      "all": true
    }
  }
}
```

### 4.1 数据库初始化 `db/database.rs`

```rust
use rusqlite::{Connection, params};
use std::sync::Mutex;

/// SQLite 数据库初始化与版本管理。
/// 职责：创建数据库文件、建表、设置版本号。
/// 依赖：rusqlite（静态编译进二进制，无 ABI 兼容问题）
///
/// v1 策略：init() 内执行 CREATE TABLE IF NOT EXISTS，
///         并设置 PRAGMA user_version = 1。
///         未来版本通过比对 user_version 执行增量迁移。
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(db_path)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// 初始化数据库，创建所有表（如不存在），设置版本号
    pub fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            -- items 表
            CREATE TABLE IF NOT EXISTS items (
                id              TEXT PRIMARY KEY,
                content         TEXT NOT NULL,
                deadline        TEXT,
                time            TEXT,
                priority        TEXT NOT NULL DEFAULT 'medium',
                type            TEXT NOT NULL DEFAULT 'other',
                remind_offsets  TEXT,
                notified_stages TEXT NOT NULL DEFAULT '[]',
                source_email_id TEXT NOT NULL,
                source_from     TEXT NOT NULL,
                source_subject  TEXT NOT NULL,
                source_account  TEXT NOT NULL,
                source_date     TEXT NOT NULL,
                matched_skill   TEXT,
                status          TEXT NOT NULL DEFAULT 'pending',
                last_notified_at TEXT,
                created_at      TEXT NOT NULL,
                completed_at    TEXT
            );

            -- accounts 表（不含密码字段，密码由 SecretStore 管理）
            CREATE TABLE IF NOT EXISTS accounts (
                id         TEXT PRIMARY KEY,
                email      TEXT NOT NULL UNIQUE,
                imap_host  TEXT NOT NULL,
                imap_port  INTEGER NOT NULL DEFAULT 993,
                last_uid   INTEGER NOT NULL DEFAULT 0,
                status     TEXT NOT NULL DEFAULT 'active'
            );

            -- skills 表（元数据，SKILL.md 内容由 SkillLoader 从文件读取）
            CREATE TABLE IF NOT EXISTS skills (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL UNIQUE,
                description TEXT NOT NULL DEFAULT '',
                enabled     INTEGER NOT NULL DEFAULT 0,
                sort_order  INTEGER NOT NULL DEFAULT 0,
                is_builtin  INTEGER NOT NULL DEFAULT 0,
                file_path   TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            );

            -- skip_list 表
            CREATE TABLE IF NOT EXISTS skip_list (
                id    TEXT PRIMARY KEY,
                type  TEXT NOT NULL,
                value TEXT NOT NULL UNIQUE
            );

            -- settings 表（key-value）
            CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            PRAGMA user_version = 3;
        ")?;

        Ok(())
    }

    /// 获取数据库连接（供 Repo 层使用）
    pub fn get_conn(&self) -> &Mutex<Connection> {
        &self.conn
    }

    /// 关闭数据库连接
    pub fn close(self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.into_inner().unwrap();
        conn.close().map_err(|e| e.1.into())
    }
}
```

### 4.2 Repository 层

> 每个 Repo 负责一张表的 CRUD，不包含业务逻辑。

#### `db/items_repo.rs`

```rust
/// 事项数据仓库。负责 items 表的增删改查。
pub struct ItemRepo {
    db: std::sync::Arc<Database>,
}

impl ItemRepo {
    pub fn new(db: std::sync::Arc<Database>) -> Self;

    /// 插入新事项，返回插入后的完整 Item
    pub fn insert(&self, item: NewItem) -> Result<Item, Box<dyn std::error::Error>>;

    /// 根据 ID 查询单个事项
    pub fn get_by_id(&self, id: &str) -> Result<Option<Item>, Box<dyn std::error::Error>>;

    /// 查询所有待办事项，按优先级升序 + deadline 升序排列
    pub fn get_pending(&self) -> Result<Vec<Item>, Box<dyn std::error::Error>>;

    /// 查询所有已完成事项，按完成时间降序排列
    pub fn get_completed(&self) -> Result<Vec<Item>, Box<dyn std::error::Error>>;

    /// 查询所有事项
    pub fn get_all(&self) -> Result<Vec<Item>, Box<dyn std::error::Error>>;

    /// 标记事项为已完成，记录完成时间
    pub fn mark_complete(&self, id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// 标记事项为已忽略
    pub fn mark_ignored(&self, id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// 将已完成/已忽略的事项恢复为待办
    pub fn restore(&self, id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// 标记事项为已过期
    pub fn mark_overdue(&self, id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// 更新事项的上次通知时间
    pub fn update_last_notified_at(&self, id: &str, timestamp: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// 查询即将到期的事项（deadline 在指定时间范围内）
    pub fn get_upcoming_deadline_items(&self, from: &str, to: &str) -> Result<Vec<Item>, Box<dyn std::error::Error>>;

    /// 更新事项的通知阶段状态
    pub fn update_notified_stages(&self, id: &str, stages: &[usize]) -> Result<(), Box<dyn std::error::Error>>;

    /// 清理超过 N 天的已完成/已忽略事项（P2-6：事项上限保护）。
    /// 返回被清理的条目数。当 completed + ignored 总数超过 MAX_COMPLETED_ITEMS 时自动调用。
    pub fn cleanup_old_completed(&self, days: u64) -> Result<u64, Box<dyn std::error::Error>>;

    /// 获取已完成 + 已忽略事项的总数（用于判断是否需要自动清理）
    pub fn get_done_count(&self) -> Result<u64, Box<dyn std::error::Error>>;
}
```

#### `db/accounts_repo.rs`

```rust
/// 邮箱账号数据仓库。负责 accounts 表的 CRUD。
/// 注意：密码不在本层处理。密码由 SecretStore 以 account:{id} 为 key，
///       通过 keyring crate 存储在操作系统原生凭证管理器中。
pub struct AccountRepo {
    db: std::sync::Arc<Database>,
}

impl AccountRepo {
    pub fn new(db: std::sync::Arc<Database>) -> Self;

    /// 插入新账号（不含密码），返回含 id 的 Account
    pub fn insert(&self, account: NewAccount) -> Result<Account, Box<dyn std::error::Error>>;

    /// 查询所有账号（不含密码）
    pub fn get_all(&self) -> Result<Vec<Account>, Box<dyn std::error::Error>>;

    /// 根据 ID 查询单个账号（不含密码）
    pub fn get_by_id(&self, id: &str) -> Result<Option<Account>, Box<dyn std::error::Error>>;

    /// 更新账号信息（不含密码）
    pub fn update(&self, id: &str, data: AccountUpdate) -> Result<(), Box<dyn std::error::Error>>;

    /// 删除账号
    pub fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// 更新账号的最后拉取 UID
    pub fn update_last_uid(&self, id: &str, uid: u64) -> Result<(), Box<dyn std::error::Error>>;

    /// 更新账号状态
    pub fn update_status(&self, id: &str, status: AccountStatus) -> Result<(), Box<dyn std::error::Error>>;
}
```

#### `db/skills_repo.rs`

```rust
/// Skill 数据仓库。负责 skills 表的 CRUD。
/// 仅存储元数据（name、description、filePath 等）。
/// SKILL.md 的文件读写由 SkillLoader 处理，不在此层。
pub struct SkillRepo {
    db: std::sync::Arc<Database>,
}

impl SkillRepo {
    pub fn new(db: std::sync::Arc<Database>) -> Self;

    /// 插入新 Skill 记录
    pub fn insert(&self, skill: NewSkill) -> Result<Skill, Box<dyn std::error::Error>>;

    /// 查询所有 Skill，按 sort_order 升序排列
    pub fn get_all(&self) -> Result<Vec<Skill>, Box<dyn std::error::Error>>;

    /// 根据 ID 查询单个 Skill
    pub fn get_by_id(&self, id: &str) -> Result<Option<Skill>, Box<dyn std::error::Error>>;

    /// 根据名称查询 Skill（名称唯一）
    pub fn get_by_name(&self, name: &str) -> Result<Option<Skill>, Box<dyn std::error::Error>>;

    /// 启用一个 Skill（单选逻辑：启用一个时，禁用其他）
    pub fn set_active(&self, id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// 禁用一个 Skill。如果禁用的是当前激活的 Skill，自动激活默认 Skill
    pub fn deactivate(&self, id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// 更新 Skill 描述
    pub fn update_description(&self, id: &str, description: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// 更新 Skill 排序顺序
    pub fn update_sort_order(&self, id: &str, sort_order: i32) -> Result<(), Box<dyn std::error::Error>>;

    /// 更新 Skill 文件路径
    pub fn update_file_path(&self, id: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// 更新 Skill 最后修改时间
    pub fn touch(&self, id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// 删除用户 Skill（内置 Skill 禁止删除，抛出错误）
    pub fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error>>;
}
```

#### `db/skip_list_repo.rs`

```rust
/// 跳过名单数据仓库。负责 skip_list 表的 CRUD。
pub struct SkipListRepo {
    db: std::sync::Arc<Database>,
}

impl SkipListRepo {
    pub fn new(db: std::sync::Arc<Database>) -> Self;

    /// 添加跳过条目
    pub fn insert(&self, entry: NewSkipEntry) -> Result<SkipEntry, Box<dyn std::error::Error>>;

    /// 查询所有跳过条目
    pub fn get_all(&self) -> Result<Vec<SkipEntry>, Box<dyn std::error::Error>>;

    /// 根据 ID 删除跳过条目
    pub fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// 判断指定发件人是否在跳过名单中
    pub fn is_sender_skipped(&self, email: &str) -> Result<bool, Box<dyn std::error::Error>>;

    /// 判断指定域名是否在跳过名单中
    pub fn is_domain_skipped(&self, domain: &str) -> Result<bool, Box<dyn std::error::Error>>;

    /// 综合判断：发件人或域名是否命中跳过名单
    pub fn is_skipped(&self, from: &str) -> Result<bool, Box<dyn std::error::Error>>;
}
```

#### `db/settings_repo.rs`

```rust
/// 通用配置数据仓库。负责 settings 表的 CRUD。
/// 以 key-value 形式存储各种设置项。
pub struct SettingsRepo {
    db: std::sync::Arc<Database>,
}

impl SettingsRepo {
    pub fn new(db: std::sync::Arc<Database>) -> Self;

    /// 获取单个配置值，不存在时返回 None
    pub fn get(&self, key: &str) -> Option<String>;

    /// 获取单个配置值，不存在时返回默认值
    pub fn get_or(&self, key: &str, default: &str) -> String;

    /// 设置单个配置值（upsert）
    pub fn set(&self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// 批量获取配置，返回 key-value 映射
    pub fn get_all(&self, keys: Option<Vec<String>>) -> Result<std::collections::HashMap<String, String>, Box<dyn std::error::Error>>;
}
```

### 4.3 安全模块 `services/secret_store.rs`

```rust
/// 凭证加密存储模块。
/// 使用 AES-256-GCM 对称加密将邮箱授权码和 API Key 加密后存入 SQLite 数据库。
/// 加密密钥在应用首次启动时随机生成，持久化到 ~/.jiuj/.key 文件。
///
/// 底层机制：
///   - 加密算法：AES-256-GCM（认证加密，防篡改）
///   - 密钥管理：首次启动自动生成 256-bit 随机密钥，存入 ~/.jiuj/.key（权限 0600）
///   - 存储方式：密文以 TEXT 形式存入 settings 表（key = "enc:{purpose}", value = ciphertext）
///
/// 存储 key 格式：
///   "enc:account:{id}"  → 邮箱授权码密文
///   "enc:ai:{provider}" → LLM API Key 密文
///
/// 依赖：aes-gcm crate, rand crate
pub struct SecretStore {
    key_path: String,
    encryption_key: Option<[u8; 32]>,
}

impl SecretStore {
    /// 初始化 SecretStore。首次启动时生成密钥并保存到 ~/.jiuj/.key
    pub fn new(app_data_dir: &str) -> Self;

    /// 加密并存储凭证到 settings 表
    pub fn set_password(&self, key: &str, value: &str, settings_repo: &SettingsRepo)
        -> Result<(), Box<dyn std::error::Error>>;

    /// 从 settings 表读取并解密凭证
    pub fn get_password(&self, key: &str, settings_repo: &SettingsRepo)
        -> Result<Option<String>, Box<dyn std::error::Error>>;

    /// 删除凭证（从 settings 表删除记录）
    pub fn delete_password(&self, key: &str, settings_repo: &SettingsRepo) -> Result<bool, Box<dyn std::error::Error>>;
}
```

### 4.4 Service 层

> 业务逻辑层，协调 Repo 和外部依赖。

#### `services/sanitizer.rs`

```rust
/// 数据脱敏 + 正文预处理服务。
/// 职责：在邮件内容发送给 LLM 之前，执行正则脱敏和正文截断。
/// 纯函数式设计，无状态，无副作用。
pub struct Sanitizer {
    rules: Vec<SanitizeRule>,
}

struct SanitizeRule {
    name: String,
    pattern: regex::Regex,
    placeholder: String,
}

impl Sanitizer {
    pub fn new() -> Self;

    /// 对文本执行脱敏处理，返回脱敏后的文本
    pub fn sanitize(&self, text: &str) -> String;

    /// 截断超长正文。超过 MAX_EMAIL_BODY_LENGTH 时，
    /// **直接从尾部截断，保留前 N 个字符**。
    /// 返回 (截断后文本, 是否被截断)
    pub fn truncate(&self, body: &str) -> (String, bool);

    /// 完整预处理流程：先脱敏再截断
    /// HTML 邮件会先去除 HTML 标签转为纯文本后再处理。
    /// 返回 (处理后文本, 是否被截断)
    pub fn process(&self, body: &str) -> (String, bool);

    /// 获取所有内置的脱敏规则列表，供 UI 展示
    pub fn get_rules(&self) -> Vec<(&str, &str, &str)>;  // (name, pattern, placeholder)
}
```

#### `services/skill_loader.rs`

```rust
/// Skill 文件加载器。
/// 职责：读取 ~/.jiuj/skills/ 目录下的 SKILL.md 文件，加载为字符串。
/// 不做任何语义解析——原始 SKILL.md 直接传给 LLM Prompt。
///
/// 内置 Skill 位于 skills/builtin/default/SKILL.md（项目目录内），
///   首次启动时复制到 ~/.jiuj/skills/default/SKILL.md。
/// 用户 Skill 位于 ~/.jiuj/skills/{name}/SKILL.md。
pub struct SkillLoader {
    skill_dir: String,
}

impl SkillLoader {
    pub fn new(skill_dir: &str) -> Self;

    /// 读取指定 Skill 的 SKILL.md 文件内容
    pub fn load_skill_content(&self, skill_name: &str) -> Option<String>;

    /// 保存 Skill 内容到文件
    pub fn save_skill_content(&self, skill_name: &str, content: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// 列出所有已安装的 Skill 目录名
    pub fn list_installed_skills(&self) -> Vec<String>;

    /// 删除指定 Skill 目录（内置 Skill 禁止删除，抛出错误）
    pub fn delete_skill(&self, skill_name: &str, is_builtin: bool) -> Result<(), Box<dyn std::error::Error>>;

    /// 创建新 Skill 目录和空 SKILL.md
    pub fn create_skill(&self, skill_name: &str, content: &str) -> Result<String, Box<dyn std::error::Error>>;

    /// 导出 Skill 为文件路径
    pub fn export_skill(&self, skill_name: &str) -> Result<String, Box<dyn std::error::Error>>;

    /// 导入 Skill（从 .md 文件复制到 skills 目录）。
    /// v1.0 不做安全扫描，直接导入。v1.1 引入基础安全扫描后返回警告列表。
    pub fn import_skill(&self, source_path: &str) -> Result<String, Box<dyn std::error::Error>>;

    /// 确保内置 default Skill 存在（首次启动时从项目目录复制）
    pub fn ensure_builtin_skill(&self, builtin_dir: &str) -> Result<(), Box<dyn std::error::Error>>;
}
```

#### `services/mail_fetcher.rs`

```rust
/// 邮件拉取服务。
/// 职责：通过 IMAP 协议定时从邮箱拉取新邮件。
/// 批量处理：一次 fetch_all() 可能拉取到多封新邮件，
///          返回所有新邮件的数组，由 AppScheduler 分批交给 AIAnalyzer 处理。
/// 依赖：imap crate, mail-parser crate, AccountRepo, SkipListRepo, SecretStore
pub struct MailFetcher {
    account_repo: AccountRepo,
    skip_list_repo: SkipListRepo,
    secret_store: SecretStore,
}

impl MailFetcher {
    pub fn new(
        account_repo: AccountRepo,
        skip_list_repo: SkipListRepo,
        secret_store: SecretStore,
    ) -> Self;

    /// 对所有 active 账号执行一次邮件拉取。
    /// 内部流程：对每个账号 → SecretStore 取密码 → IMAP 连接 → 按 last_uid 拉取新邮件 →
    ///           跳过名单过滤 → 解析邮件正文 → 更新 last_uid
    pub async fn fetch_all(&self) -> Result<Vec<RawEmail>, Box<dyn std::error::Error>>;

    /// 对单个账号执行邮件拉取
    pub async fn fetch_by_account(&self, account_id: &str) -> Result<Vec<RawEmail>, Box<dyn std::error::Error>>;

    /// 测试单个邮箱账号的 IMAP 连接
    pub async fn test_connection(&self, account_id: &str) -> Result<bool, Box<dyn std::error::Error>>;
}
```

#### `services/ai_analyzer.rs`

```rust
/// AI 分析服务。
/// 职责：批量拼接 Prompt（系统模板 + Skill 原文 + 邮件 JSON 数组）→ 调用 LLM → 解析结果。
///
/// LLM 调用规范：
///   - 所有厂商统一使用 OpenAI Chat Completions API 格式
///   - 使用 reqwest（Rust HTTP 客户端），无 Node.js 依赖
///   - 请求格式：POST {baseUrl}/chat/completions
///   - 超时时间：30 秒（tokio::time::timeout）
///   - 不做单次重试，批次失败由 AppScheduler 统一管理重试
///
/// 错误分类策略（PRD P1-2）：
///   - 网络类错误（超时/连接/DNS）→ 返回 RetryableError，AppScheduler 可重试
///   - 认证/配额类错误（401/429）→ 返回 FatalError，立即停止本轮，通知前端
///   - 解析类错误（JSON 无效）→ 返回空 Vec，记日志，不重试
///
/// 依赖：Sanitizer, SkillLoader, SkillRepo, SettingsRepo, SecretStore
pub struct AIAnalyzer {
    sanitizer: Sanitizer,
    skill_loader: SkillLoader,
    skill_repo: SkillRepo,
    settings_repo: SettingsRepo,
    secret_store: SecretStore,
    client: reqwest::Client,
}

/// 批次分析结果：区分可重试错误和致命错误
#[derive(Debug)]
pub enum BatchResult {
    /// 成功：提取到的事项列表
    Success(Vec<ExtractedItem>),
    /// 可重试错误：网络超时、连接失败等（HTTP 5xx / 超时 / DNS 失败）
    Retryable(String),
    /// 致命错误：认证失败、配额耗尽等（401 / 429），不应重试
    Fatal(String),
}

impl AIAnalyzer {
    pub fn new(
        sanitizer: Sanitizer,
        skill_loader: SkillLoader,
        skill_repo: SkillRepo,
        settings_repo: SettingsRepo,
        secret_store: SecretStore,
    ) -> Self;

    /// 批量分析邮件，提取事项。每次最多处理 10 封。
    /// 内部流程：
    ///   1. 从 SkillRepo 获取当前启用的 Skill
    ///   2. 通过 SkillLoader.load_skill_content() 读取 SKILL.md 原文
    ///   3. 对每封邮件 body 执行 Sanitizer.process()（脱敏+截断）
    ///   4. 格式化为固定 JSON 结构（from/subject/date/body/accountId/truncated）
    ///   5. 拼接系统 Prompt + Skill 原文 + 邮件 JSON 数组
    ///   6. 调用 LLM API（一次 reqwest，30 秒超时）
    ///   7. 解析 JSON 响应为 Vec<ExtractedItem>（解析失败时返回 BatchResult::Success(vec![]) 并记录日志）
    pub async fn analyze_batch(&self, emails: &[RawEmail]) -> Result<BatchResult, Box<dyn std::error::Error>>;

    /// 测试 AI 连接是否正常
    pub async fn test_connection(&self) -> Result<bool, Box<dyn std::error::Error>>;

    /// 判断 HTTP 状态码是否为可重试错误
    fn is_retryable_error(&self, status_code: u16) -> bool;
}
```

**LLM API 调用实现要点：**

```rust
// AIAnalyzer 内部 — reqwest 调用伪代码
async fn call_llm(&self, system_prompt: &str, user_messages: &str) -> Result<String, Box<dyn std::error::Error>> {
    let config = self.get_ai_config()?;
    let base_url = self.get_base_url(&config);
    let url = format!("{}/chat/completions", base_url);

    let body = serde_json::json!({
        "model": config.model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_messages },
        ],
        "temperature": 0.1,
    });

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(LLM_TIMEOUT_SECS),
        self.client.post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", config.api_key))
            .json(&body)
            .send()
    ).await??;

    let data: serde_json::Value = result.json().await?;
    let content = data["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Invalid LLM response")?;
    Ok(content.to_string())
}
```

#### `services/reminder_engine.rs`

```rust
/// 提醒引擎。
/// 职责：定时检查待办事项，在合适时机发送桌面通知。
/// 采用两段式提醒：默认提前 1 天 + 提前 2 小时。
/// 支持过期边界处理、免打扰时段判断。
/// 依赖：ItemRepo, SettingsRepo, tauri::AppHandle（通知发送）
pub struct ReminderEngine {
    item_repo: ItemRepo,
    settings_repo: SettingsRepo,
    app_handle: Option<tauri::AppHandle>,
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl ReminderEngine {
    pub fn new(item_repo: ItemRepo, settings_repo: SettingsRepo) -> Self;

    /// 注入 Tauri AppHandle（用于发送桌面通知和前端推送）
    pub fn set_app_handle(&mut self, handle: tauri::AppHandle);

    /// 启动定时检查（内部 tokio::time::interval）
    pub fn start(&self, interval_secs: u64);

    /// 停止定时检查
    pub fn stop(&self);

    /// 对新提取的事项发送通知。
    /// 根据 notify-rules 决定是否发送、是否在免打扰时段内
    pub fn notify_new_items(&self, items: &[Item]) -> Result<(), Box<dyn std::error::Error>>;

    /// 检查即将到期的事项，发送到期提醒（两段式）。
    ///
    /// 对每个未完成且有截止时间的事项：
    ///   1. 获取该事项的 remind_offsets（Skill 定义 > 用户默认 > 系统兜底 [1440, 120]）
    ///   2. 计算每个阶段的提醒时间点：deadline - offset
    ///   3. 过期阶段合并为一条通知（不连续弹窗）
    ///   4. 如果提醒时间已到且未通知 → 发送通知
    ///   5. 如果提醒时间已过且未通知 → 加入过期队列
    ///   6. 仅第一段过期+第二段未到 → 第一段立即通知，第二段按时通知
    ///   7. 两段都过期 → 合并为一次立即通知
    ///   8. 已截止事项（deadline <= now）→ 标记为 overdue，不触发提醒
    ///   9. 无截止时间事项 → 不触发到期提醒
    ///  10. remind_offsets 为 Some([]) → 不触发到期提醒
    pub fn check_deadlines(&self) -> Result<(), Box<dyn std::error::Error>>;

    /// 标记已过期事项（截止时间到达且未完成/未忽略）
    pub fn mark_overdue(&self) -> Result<(), Box<dyn std::error::Error>>;

    /// 获取用户设置的默认提醒偏移量。
    /// 优先级：用户界面设置 > 系统兜底 [1440, 120]
    pub fn get_default_remind_offsets(&self) -> Vec<u64>;
}
```

#### `services/tray_manager.rs`

```rust
/// 系统托盘管理。
/// 职责：创建托盘图标、右键菜单、状态切换。
/// 依赖：Tauri Tray API, tauri-plugin-notification
///
/// 注：Tauri v2 的系统托盘通过 tauri.conf.json + Rust 代码配合实现，
///     不需要单独的 TrayManager 类。此模块仅为逻辑组织。
pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // 托盘菜单项：
    //   - "打开看板" → app.get_window("main").unwrap().show()
    //   - "暂停拉取" / "恢复拉取" → app_scheduler.stop_fetch() / start_fetch()
    //   - "退出" → app.exit(0)（停止所有后台任务）
    //
    // 状态指示：
    //   normal  → 默认图标
    //   paused  → 灰色图标
    //   error   → 红色图标
    Ok(())
}

/// 更新托盘图标状态
pub fn set_tray_status(app: &tauri::AppHandle, status: TrayStatus);
```

#### `services/app_scheduler.rs`

```rust
/// 应用调度器。
/// 职责：管理邮件拉取定时任务，以及批量邮件的处理流水线。
///
/// 每次定时触发后执行完整流水线：
///   1. MailFetcher.fetch_all() 拉取新邮件（已跳过名单过滤）
///   2. 按 MAX_BATCH_SIZE（10 封）分批，每批调用 AIAnalyzer.analyze_batch()
///   3. 每批结果直接存入数据库
///   4. 汇总本轮新增事项，交给 ReminderEngine.notify_new_items() 合并通知
///   5. 通过 app_handle.emit() 推送给前端
/// 注：脱敏和格式化由 AIAnalyzer.analyze_batch() 内部完成，AppScheduler 不重复处理
pub struct AppScheduler {
    mail_fetcher: MailFetcher,
    ai_analyzer: AIAnalyzer,
    reminder_engine: ReminderEngine,
    item_repo: ItemRepo,
    app_handle: Option<tauri::AppHandle>,
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
    interval_minutes: std::sync::Arc<std::sync::Mutex<u64>>,
}

impl AppScheduler {
    pub fn new(
        mail_fetcher: MailFetcher,
        ai_analyzer: AIAnalyzer,
        reminder_engine: ReminderEngine,
        item_repo: ItemRepo,
    ) -> Self;

    /// 注入 Tauri AppHandle（用于向前端推送新事项）
    pub fn set_app_handle(&mut self, handle: tauri::AppHandle);

    /// 启动邮件拉取定时任务
    pub fn start_fetch(&self, interval_minutes: u64);

    /// 停止邮件拉取定时任务
    pub fn stop_fetch(&self);

    /// 更新拉取频率（停止旧定时器，启动新定时器）
    pub fn update_interval(&self, interval_minutes: u64);

    /// 获取当前是否正在拉取
    pub fn is_running(&self) -> bool;

    /// 手动触发一次全量重新扫描（用户更换 Skill 后主动触发）。
    /// 重新从所有 active 账号拉取最近 N 封邮件（默认 N=50，可配置），
    /// 跳过 UID 断点续拉。**不做去重——同一事件反复发邮件说明重要，值得重复提醒。**
    /// 新事项正常入库，与已有事项可能内容相似。
    pub async fn rescan_recent(&self, limit_per_account: Option<u64>) -> Result<Vec<Item>, Box<dyn std::error::Error>>;

    /// 执行一次完整的拉取 → 分析 → 通知 → 推送流水线。
    /// 内部流程（PRD P1-2 错误分类策略）：
    ///   1. 调用 mail_fetcher.fetch_all() 获取新邮件
    ///   2. 按 MAX_BATCH_SIZE 分批，每批调用 ai_analyzer.analyze_batch()
    ///   3. 对 BatchResult::Success：将事项存入数据库
    ///   4. 对 BatchResult::Retryable：记录到 retryable_batches，继续下一批
    ///   5. 对 BatchResult::Fatal：
    ///      a. 立即停止后续批次处理
    ///      b. 通过 app_handle.emit("ai:fatal-error", message) 推送给前端
    ///      c. 更新托盘图标为 error 状态
    ///      d. 返回已成功处理的事项（已存储的不回滚）
    ///   6. 所有非致命批次完成后，对 retryable_batches 中的批次逐一重试（最多 1 次）
    ///   7. 单封邮件连续两次导致 Retryable 的，自动隔离该邮件并记录日志
    ///   8. 汇总所有新增事项，调用 reminder_engine.notify_new_items() 合并通知
    ///   9. 通过 app_handle.emit("fetch:new-items", new_items) 推送给前端
    pub async fn run_once(&self) -> Result<Vec<Item>, Box<dyn std::error::Error>>;
}
```

### 4.5 日志规范

> 使用 `log` + `env_logger`，按级别分类输出。

| 级别 | 用途 | 示例场景 |
|------|------|---------|
| error | 错误，需关注 | LLM 调用失败、数据库写入失败、IMAP 连接失败 |
| warn | 警告 | 邮件解析异常（跳过该邮件）、批次重试、Skill 文件不存在 |
| info | 关键业务事件 | 定时拉取完成、新事项入库、Skill 切换、应用启动/退出 |
| debug | 调试信息 | Tauri command 调用参数、SQL 查询、脱敏规则匹配详情 |

```rust
// 使用方式（Rust 后端）
use log::{info, error, warn};

info!("邮件拉取完成: 账号数={}, 新邮件={}", account_count, new_count);
error!("LLM 调用失败: error={}, 批次={}", err, batch_num);
warn!("邮件解析异常，跳过: email_id={}, reason={}", id, reason);
```

**日志文件位置：**
- macOS: `~/Library/Logs/{bundle_identifier}/`
- Windows: `%USERPROFILE%\AppData\Roaming\{bundle_identifier}\logs\`

---

## 5. Tauri IPC — Controller 层

> Tauri 的 `#[tauri::command]` 宏替代了 Electron 的 ipcMain.handle()。
> 每个 command 是一个 Rust 函数，编译期类型检查，不存在通道名拼写错误的风险。

### 设计原则

- **类型安全**：所有参数和返回值都是 Rust 类型，通过 serde 自动序列化/反序列化
- **无通道字符串**：不需要定义 IPC 通道常量，函数名即通道
- **自动错误处理**：command 返回 `Result<T, E>` 时，E 自动序列化为前端可读的错误信息
- **状态注入**：通过 `tauri::State` 自动注入 Repo 和 Service 实例

### Command 注册

```rust
// src-tauri/src/commands/mod.rs

// ── 事项 ──
pub async fn get_all_items(item_repo: tauri::State<'_, ItemRepo>) -> Result<Vec<Item>, String>;
pub async fn get_pending_items(item_repo: tauri::State<'_, ItemRepo>) -> Result<Vec<Item>, String>;
pub async fn get_completed_items(item_repo: tauri::State<'_, ItemRepo>) -> Result<Vec<Item>, String>;
pub async fn complete_item(id: String, item_repo: tauri::State<'_, ItemRepo>) -> Result<(), String>;
pub async fn ignore_item(id: String, item_repo: tauri::State<'_, ItemRepo>) -> Result<(), String>;
pub async fn restore_item(id: String, item_repo: tauri::State<'_, ItemRepo>) -> Result<(), String>;
pub async fn get_item_detail(id: String, item_repo: tauri::State<'_, ItemRepo>) -> Result<Item, String>;

// ── 拉取 ──
pub async fn start_fetch(scheduler: tauri::State<'_, AppScheduler>) -> Result<(), String>;
pub async fn stop_fetch(scheduler: tauri::State<'_, AppScheduler>) -> Result<(), String>;
pub async fn get_fetch_status(scheduler: tauri::State<'_, AppScheduler>) -> Result<bool, String>;
pub async fn set_fetch_interval(minutes: u64, scheduler: tauri::State<'_, AppScheduler>) -> Result<(), String>;
/// 手动触发全量重扫（更换 Skill 后使用）
pub async fn rescan_recent(scheduler: tauri::State<'_, AppScheduler>) -> Result<Vec<Item>, String>;

// ── 账号 ──
pub async fn get_all_accounts(account_repo: tauri::State<'_, AccountRepo>) -> Result<Vec<Account>, String>;
/// 添加邮箱账号：
///   1. account_repo.insert(email, host, port) → 返回 Account（含 id）
///   2. secret_store.set_password(format!("enc:account:{}", account.id), password, &settings_repo)
    data: AddAccountRequest,
    account_repo: tauri::State<'_, AccountRepo>,
    secret_store: tauri::State<'_, SecretStore>,
) -> Result<Account, String>;
/// 更新邮箱账号：
///   1. 如果 password 字段有值 → secret_store.set_password(format!("enc:account:{}", id), password, &settings_repo)
///   2. account_repo.update(id, { email, host, port })
pub async fn update_account(
    id: String,
    data: UpdateAccountRequest,
    account_repo: tauri::State<'_, AccountRepo>,
    secret_store: tauri::State<'_, SecretStore>,
) -> Result<(), String>;
/// 删除邮箱账号：
///   1. secret_store.delete_password(format!("enc:account:{}", id), &settings_repo)
///   2. account_repo.delete(id)
pub async fn delete_account(
    id: String,
    account_repo: tauri::State<'_, AccountRepo>,
    secret_store: tauri::State<'_, SecretStore>,
) -> Result<(), String>;
/// 测试账号连接
pub async fn test_account_connection(
    id: String,
    mail_fetcher: tauri::State<'_, MailFetcher>,
) -> Result<bool, String>;

// ── AI 设置 ──
pub async fn get_ai_config(
    settings_repo: tauri::State<'_, SettingsRepo>,
    secret_store: tauri::State<'_, SecretStore>,
) -> Result<AIConfig, String>;
/// 设置 AI 配置：
///   1. settings_repo.set("aiProvider", config.provider)
///   2. settings_repo.set("aiModel", config.model)
///   3. secret_store.set_password(format!("enc:ai:{}", config.provider), config.api_key, &settings_repo)
pub async fn set_ai_config(
    config: AIConfig,
    settings_repo: tauri::State<'_, SettingsRepo>,
    secret_store: tauri::State<'_, SecretStore>,
) -> Result<(), String>;
pub async fn test_ai_connection(
    ai_analyzer: tauri::State<'_, AIAnalyzer>,
) -> Result<bool, String>;

// ── Skill ──
pub async fn get_all_skills(skill_repo: tauri::State<'_, SkillRepo>) -> Result<Vec<Skill>, String>;
pub async fn get_active_skill(skill_repo: tauri::State<'_, SkillRepo>) -> Result<Option<Skill>, String>;
pub async fn set_active_skill(id: String, skill_repo: tauri::State<'_, SkillRepo>) -> Result<(), String>;
pub async fn deactivate_skill(id: String, skill_repo: tauri::State<'_, SkillRepo>) -> Result<(), String>;
pub async fn get_skill_content(name: String, skill_loader: tauri::State<'_, SkillLoader>) -> Result<Option<String>, String>;
pub async fn save_skill_content(
    name: String, content: String,
    skill_loader: tauri::State<'_, SkillLoader>,
    skill_repo: tauri::State<'_, SkillRepo>,
) -> Result<(), String>;
pub async fn create_skill(
    name: String, description: String,
    skill_loader: tauri::State<'_, SkillLoader>,
    skill_repo: tauri::State<'_, SkillRepo>,
) -> Result<Skill, String>;
pub async fn delete_skill(
    id: String,
    skill_loader: tauri::State<'_, SkillLoader>,
    skill_repo: tauri::State<'_, SkillRepo>,
) -> Result<(), String>;
pub async fn update_skill_sort_order(id: String, sort_order: i32, skill_repo: tauri::State<'_, SkillRepo>) -> Result<(), String>;
pub async fn reset_skill(name: String, skill_loader: tauri::State<'_, SkillLoader>) -> Result<(), String>;
pub async fn export_skill(name: String, skill_loader: tauri::State<'_, SkillLoader>) -> Result<String, String>;
pub async fn import_skill(
    source_path: String,
    skill_loader: tauri::State<'_, SkillLoader>,
    skill_repo: tauri::State<'_, SkillRepo>,
) -> Result<String, String>;  // skill_name

// ── 跳过名单 ──
pub async fn get_all_skip_entries(skip_repo: tauri::State<'_, SkipListRepo>) -> Result<Vec<SkipEntry>, String>;
pub async fn add_skip_entry(
    entry: NewSkipEntry,
    skip_repo: tauri::State<'_, SkipListRepo>,
) -> Result<SkipEntry, String>;
pub async fn delete_skip_entry(id: String, skip_repo: tauri::State<'_, SkipListRepo>) -> Result<(), String>;
/// 从事项添加到跳过名单：
///   1. item_repo.get_by_id(item_id) → 获取 source_from
///   2. skip_repo.insert({ type: sender, value: source_from })
pub async fn add_skip_from_item(
    item_id: String,
    item_repo: tauri::State<'_, ItemRepo>,
    skip_repo: tauri::State<'_, SkipListRepo>,
) -> Result<(), String>;

// ── 通用设置 ──
pub async fn get_setting(key: String, settings_repo: tauri::State<'_, SettingsRepo>) -> Result<Option<String>, String>;
/// 设置配置值（upsert）。
/// 特殊 key 处理：
///   - key === "autoStart" → 通过 tauri-plugin-autostart 管理开机启动
pub async fn set_setting(key: String, value: String, settings_repo: tauri::State<'_, SettingsRepo>) -> Result<(), String>;
pub async fn get_all_settings(
    keys: Option<Vec<String>>,
    settings_repo: tauri::State<'_, SettingsRepo>,
) -> Result<std::collections::HashMap<String, String>, String>;
```

---

## 6. View 层 — WebView 前端

> 位于 `src/`，使用 React + TypeScript + Tailwind CSS + Radix UI 构建用户界面。
> 所有与 Rust 后端的通信通过 `@tauri-apps/api` 的 `invoke` 函数调用。

### 6.1 技术选型细节

| 职责 | 选型 | 说明 |
|------|------|------|
| 路由 | react-router-dom v6 | HashRouter（桌面应用无需服务端路径） |
| 数据获取 | TanStack Query v5 | 封装 invoke 调用，自动缓存 + `listen` 推送触发 refetch |
| 表单 | react-hook-form v7 | 账号表单、AI 配置表单、Skill 编辑器 |
| UI 组件 | Radix UI | Dialog（确认弹窗）、Select（下拉选择）、Switch（开关）等 |
| CSS | Tailwind CSS v3 | JIT 模式，自定义主题色 |
| 图标 | lucide-react | 轻量 SVG 图标库 |
| Tauri API | @tauri-apps/api | invoke（调用 Rust command）、listen（监听 Rust 推送） |

### 6.2 Tauri API 封装 `lib/tauri-api.ts`

```typescript
/**
 * Tauri invoke 的类型安全封装。
 * 所有前端对后端的调用都通过此文件，确保参数和返回值类型正确。
 */
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

// 类型定义（与 Rust models.rs 对齐）
type Priority = 'high' | 'medium' | 'low';
type ItemType = 'task' | 'deadline' | 'reply' | 'notification' | 'other';
type ItemStatus = 'pending' | 'completed' | 'ignored' | 'overdue';
type AccountStatus = 'active' | 'disabled';
type SkipType = 'sender' | 'domain';
type TrayStatus = 'normal' | 'paused' | 'error';
type AIProvider = 'openai' | 'deepseek' | 'kimi' | 'zhipu' | 'qwen' | 'claude' | 'ollama' | 'custom';
type Locale = 'zh-CN' | 'en-US' | 'auto';
type CloseAction = 'minimize-to-tray' | 'quit';

interface Item { /* 同 PRD 定义，含 attachments 字段 */ }
interface Account { /* 同 PRD 定义 */ }
interface Skill { /* 同 PRD 定义 */ }
interface SkipEntry { /* 同 PRD 定义 */ }
interface AIConfig { /* 同 PRD 定义 */ }
interface SkillTemplate { /* 同 PRD 定义 */ }

// ── 事项 ──
export const api = {
  items: {
    getAll:     () => invoke<Item[]>('get_all_items'),
    getPending: () => invoke<Item[]>('get_pending_items'),
    getCompleted:() => invoke<Item[]>('get_completed_items'),
    complete:   (id: string) => invoke<void>('complete_item', { id }),
    ignore:     (id: string) => invoke<void>('ignore_item', { id }),
    restore:    (id: string) => invoke<void>('restore_item', { id }),
    getDetail:  (id: string) => invoke<Item>('get_item_detail', { id }),
    /** 监听后台推送的新事项（Rust → 前端），返回取消监听函数 */
    onNewItems: (callback: (items: Item[]) => void): Promise<UnlistenFn> => {
      return listen<Item[]>('fetch:new-items', (event) => callback(event.payload));
    },
  },

  fetch: {
    start:     () => invoke<void>('start_fetch'),
    stop:      () => invoke<void>('stop_fetch'),
    getStatus: () => invoke<boolean>('get_fetch_status'),
    setInterval: (minutes: number) => invoke<void>('set_fetch_interval', { minutes }),
    /** 手动触发全量重扫（更换 Skill 后使用） */
    rescan:   (limitPerAccount?: number) => invoke<Item[]>('rescan_recent', { limitPerAccount }),

  },

  accounts: {
    getAll:  () => invoke<Account[]>('get_all_accounts'),
    add:     (data: { email: string; imapHost: string; imapPort: number; password: string }) =>
               invoke<Account>('add_account', { data }),
    update:  (id: string, data: Partial<{ email: string; imapHost: string; imapPort: number; password: string }>) =>
               invoke<void>('update_account', { id, data }),
    delete:  (id: string) => invoke<void>('delete_account', { id }),
    test:    (id: string) => invoke<boolean>('test_account_connection', { id }),
  },

  ai: {
    getConfig:    () => invoke<AIConfig>('get_ai_config'),
    setConfig:    (config: AIConfig) => invoke<void>('set_ai_config', { config }),
    testConnection: () => invoke<boolean>('test_ai_connection'),
  },

  skill: {
    getAll:      () => invoke<Skill[]>('get_all_skills'),
    getActive:   () => invoke<Skill | null>('get_active_skill'),
    setActive:   (id: string) => invoke<void>('set_active_skill', { id }),
    deactivate:  (id: string) => invoke<void>('deactivate_skill', { id }),
    getContent:  (name: string) => invoke<string | null>('get_skill_content', { name }),
    saveContent: (name: string, content: string) => invoke<void>('save_skill_content', { name, content }),
    create:      (name: string, description: string) => invoke<Skill>('create_skill', { name, description }),
    delete:      (id: string) => invoke<void>('delete_skill', { id }),
    updateSortOrder: (id: string, sortOrder: number) => invoke<void>('update_skill_sort_order', { id, sortOrder }),
    reset:       (name: string) => invoke<void>('reset_skill', { name }),
    export:      (name: string) => invoke<string>('export_skill', { name }),
    /** 导入 Skill，返回 skillName。v1.0 不做安全扫描 */
    import:      (sourcePath: string) => invoke<string>('import_skill', { sourcePath }),
  },

  skip: {
    getAll:       () => invoke<SkipEntry[]>('get_all_skip_entries'),
    add:          (entry: { skipType: SkipType; value: string }) => invoke<SkipEntry>('add_skip_entry', { entry }),
    delete:       (id: string) => invoke<void>('delete_skip_entry', { id }),
    addFromItem:  (itemId: string) => invoke<void>('add_skip_from_item', { itemId }),
  },

  settings: {
    get:     (key: string) => invoke<string | null>('get_setting', { key }),
    set:     (key: string, value: string) => invoke<void>('set_setting', { key, value }),
    getAll:  (keys?: string[]) => invoke<Record<string, string>>('get_all_settings', { keys }),
  },
};

// Window.api 类型声明（供全局使用）
declare global {
  const api: typeof api;
}
```

### 6.3 页面结构

| 页面 | 路径 | 功能 |
|------|------|------|
| Board | `pages/BoardPage.tsx` | 看板，展示所有事项（含空状态 + AI 异常提示条） |
| Accounts | `pages/AccountsPage.tsx` | 邮箱管理 |
| AISettings | `pages/AISettingsPage.tsx` | AI 设置 |
| Skills | `pages/SkillsPage.tsx` | Skill 管理（单选 Radio 模式） |
| SkillEditor | `pages/SkillEditorPage.tsx` | Skill 编辑器 |
| SkipList | `pages/SkipListPage.tsx` | 跳过名单 |
| General | `pages/GeneralPage.tsx` | 通用设置 |

### 6.4 页面组件

#### `pages/BoardPage.tsx` — 看板页

```typescript
/**
 * 看板主页面（函数组件）。
 * 功能：展示待办/已完成事项列表，支持完成/忽略/快捷跳过/查看详情操作。
 * 数据源：api.items.getPending / getCompleted
 * 推送：api.items.onNewItems → 自动 refetch

 */
function BoardPage() {


  const { data: pendingItems = [] } = useQuery({
    queryKey: ['items', 'pending'],
    queryFn: () => api.items.getPending(),
  });
  const { data: completedItems = [] } = useQuery({
    queryKey: ['items', 'completed'],
    queryFn: () => api.items.getCompleted(),
  });

  // 监听后台推送的新事项，自动刷新列表
  useEffect(() => {
    let unlisten: UnlistenFn;

    api.items.onNewItems(() => {
      queryClient.invalidateQueries({ queryKey: ['items'] });
    }).then(fn => unlisten = fn);

    return () => { unlisten?.(); };
  }, []);

  const handleComplete = async (id: string) => {
    await api.items.complete(id);
    queryClient.invalidateQueries({ queryKey: ['items'] });
  };

  const handleIgnore = async (id: string) => {
    await api.items.ignore(id);
    queryClient.invalidateQueries({ queryKey: ['items'] });
  };

  const handleAddToSkipList = async (itemId: string) => {
    // 弹出 ConfirmDialog 确认后执行
    await api.skip.addFromItem(itemId);
    queryClient.invalidateQueries({ queryKey: ['items'] });
  };

  const handleItemClick = (item: Item) => {
    // 打开 ItemDetailDialog
  };

  // 空状态渲染（P2-8）
  if (pendingItems.length === 0 && completedItems.length === 0) {
    return <BoardEmptyState />;
  }

  return (
    <>

      {/* 正常看板内容 */}
    </>
  );
}
```

#### `pages/AccountsPage.tsx` — 邮箱设置页

```typescript
/**
 * 邮箱账号管理页面（函数组件）。
 * 功能：添加/编辑/删除邮箱账号，配置拉取频率。
 * 数据源：api.accounts.getAll / api.settings
 */
function AccountsPage() {
  const { data: accounts = [], refetch } = useQuery({
    queryKey: ['accounts'],
    queryFn: () => api.accounts.getAll(),
  });

  const handleAddAccount = async (data: AccountFormData) => {
    await api.accounts.add(data);
    refetch();
  };

  const handleDeleteAccount = async (id: string) => {
    // 弹出 ConfirmDialog 确认
    await api.accounts.delete(id);
    refetch();
  };

  const handleTestConnection = async (id: string) => {
    const ok = await api.accounts.test(id);
    // 显示连接结果
  };

  const handleChangeFetchInterval = async (minutes: number) => {
    await api.fetch.setInterval(minutes);
  };
}

interface AccountFormData {
  email: string;
  imapHost: string;
  imapPort: number;
  password: string;
}
```

#### `pages/AISettingsPage.tsx` — AI 设置页

```typescript
/**
 * AI 模型配置页面（函数组件）。
 * 功能：选择厂商、输入 API Key、测试连接。
 * 数据源：api.ai.getConfig
 */
function AISettingsPage() {
  const { data: config } = useQuery({
    queryKey: ['ai-config'],
    queryFn: () => api.ai.getConfig(),
  });

  const handleChangeProvider = (provider: AIProvider) => {
    // 从 AI_PROVIDERS 获取 models 列表和 recommendedModel，自动填充推荐模型
  };

  const handleSubmitConfig = async (config: AIConfig) => {
    await api.ai.setConfig(config);
  };

  const handleTestConnection = async () => {
    const ok = await api.ai.testConnection();
  };
}
```

#### `pages/SkillsPage.tsx` — Skills 设置页

```typescript
/**
 * Skills 管理页面（函数组件）。
 * 功能：浏览/新建/导入/导出 Skill，单选切换当前启用的 Skill。
 * v1.0 仅包含一个内置 default Skill，用户可创建自定义 Skill。
 * 数据源：api.skill.getAll / getActive
 */
function SkillsPage() {
  const { data: skills = [] } = useQuery({
    queryKey: ['skills'],
    queryFn: () => api.skill.getAll(),
  });
  const { data: activeSkill } = useQuery({
    queryKey: ['skills', 'active'],
    queryFn: () => api.skill.getActive(),
  });

  const handleSelectSkill = async (id: string) => {
    await api.skill.setActive(id);
    // refetch active skill
  };

  // 内置 Skill: 跳转到 SkillEditorPage(isBuiltin=true, readonly)，按钮文字为"查看"
  // 用户 Skill: 跳转到 SkillEditorPage(isBuiltin=false)，按钮文字为"编辑"

  const handleCreateSkill = async (name: string, description: string) => {
    await api.skill.create(name, description);
    // refetch skills
  };

  const handleImportSkill = async (sourcePath: string) => {
    await api.skill.import(sourcePath);
    // refetch skills
  };
}
```

#### `pages/SkillEditorPage.tsx` — Skill 编辑器

```typescript
/**
 * Skill 模板填空编辑器（函数组件）。
 * 功能：将 SKILL.md 拆分为 5 个段落（identity / extract-rules / priority-rules
 *       / notify-rules / custom-prompt），用户分段填写后保存。
 *
 * 段落解析方式：按 `## section-name` 做简单的 Markdown 标题匹配，
 *   仅用于 UI 编辑器的结构化展示，不影响 LLM 管线（LLM 管线直接读取完整 SKILL.md 原文）。
 *
 * 内置 Skill：只读模式（readonly=true），不可编辑。
 */
function SkillEditorPage({ skillName, isBuiltin }: { skillName: string; isBuiltin: boolean }) {
  const { data: content } = useQuery({
    queryKey: ['skill-content', skillName],
    queryFn: () => api.skill.getContent(skillName),
  });

  const handleSave = async (template: SkillTemplate) => {
    // 将 template.sections 拼接为完整 SKILL.md 格式文本
    const md = buildSkillMarkdown(template);
    await api.skill.saveContent(skillName, md);
  };

  const handleReset = async () => {
    // 弹出 ConfirmDialog 确认
    await api.skill.reset(skillName);
  };
}
```

#### `pages/SkipListPage.tsx` — 跳过名单页

```typescript
/**
 * 跳过名单管理页面（函数组件）。
 * 功能：添加/删除跳过的发件人和域名。
 * 数据源：api.skip.getAll
 */
function SkipListPage() {
  const { data: entries = [], refetch } = useQuery({
    queryKey: ['skip-list'],
    queryFn: () => api.skip.getAll(),
  });

  const handleAddSender = async (value: string) => {
    await api.skip.add({ skipType: 'sender', value });
    refetch();
  };

  const handleAddDomain = async (value: string) => {
    await api.skip.add({ skipType: 'domain', value });
    refetch();
  };

  const handleDelete = async (id: string) => {
    await api.skip.delete(id);
    refetch();
  };
}
```

#### `pages/GeneralPage.tsx` — 通用设置页

```typescript
/**
 * 通用设置页面（函数组件）。
 * 功能：语言切换、开机启动、关闭行为、默认提醒时间、免打扰时段。
 * 数据源：api.settings.getAll
 */
function GeneralPage() {
  const { data: settings = {} } = useQuery({
    queryKey: ['settings'],
    queryFn: () => api.settings.getAll(),
  });

  const handleChangeLanguage = async (locale: Locale) => {
    i18next.changeLanguage(locale === 'auto' ? navigator.language : locale);
    await api.settings.set('locale', locale);
  };

  const handleChangeCloseAction = async (action: CloseAction) => {
    await api.settings.set('closeAction', action);
  };

  const handleChangeRemindOffsets = async (offsets: number[]) => {
    await api.settings.set('remindOffsets', JSON.stringify(offsets));
  };
}
```

### 6.5 通用组件

#### `components/ItemCard.tsx` — 事项卡片

```typescript
interface ItemCardProps {
  item: Item;
  onComplete: (id: string) => void;
  onIgnore: (id: string) => void;
  onRestore?: (id: string) => void;
  onAddToSkipList: (itemId: string) => void;
  onClick: (item: Item) => void;
  isCompleted?: boolean;
}
```

#### `components/ItemDetailDialog.tsx` — 事项详情弹窗

```typescript
/**
 * 事项详情弹窗（基于 Radix UI Dialog）。
 * 展示来源邮件摘要、提取的关键信息、命中的 Skill。
 * PRD 4.3：点击事项可查看来源邮件摘要、提取的关键信息、命中的 Skill。
 */
interface ItemDetailDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  item: Item | null;
}
```

#### `components/ConfirmDialog.tsx` — 确认弹窗

```typescript
/**
 * 通用确认弹窗组件（基于 Radix UI Dialog）。
 * 用于"删除账号""加入跳过名单""重置 Skill"等需要二次确认的操作。
 */
interface ConfirmDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  message: string;
  confirmText?: string;   // 默认 t('common.confirm')
  cancelText?: string;    // 默认 t('common.cancel')
  onConfirm: () => void;
  variant?: 'danger' | 'default';  // 危险操作用红色确认按钮
}
```

#### `components/SkillTemplateEditor.tsx` — Skill 模板编辑器

```typescript
/**
 * Skill 模板填空编辑器组件。
 * 将 SKILL.md 拆分为 identity / extract-rules / priority-rules / notify-rules
 *   / custom-prompt 五个可编辑区域。
 *
 * 段落拆分逻辑：按 Markdown 二级标题 `## section-name` 分割。
 * 这仅用于 UI 编辑器的结构化展示，与 LLM 管线无关。
 */
interface SkillTemplateEditorProps {
  template: SkillTemplate;
  readonly?: boolean;
  onChange: (template: SkillTemplate) => void;
}
```

#### `components/BoardEmptyState.tsx` — 看板空状态（P2-8）

```typescript
/**
 * 看板空状态组件（PRD §5.9）。
 * 首次安装或所有事项已清理时展示。
 * 引导用户前往邮箱设置开始使用。
 */
function BoardEmptyState() {
  return (
    <div className="flex flex-col items-center justify-center h-full text-center p-8">
      <div className="text-6xl mb-4">🐦</div>
      <h2 className="text-xl font-medium mb-2">{t('board.empty.title')}</h2>
      <p className="text-gray-500 mb-6">{t('board.empty.description')}</p>
      <button onClick={() => window.location.hash = '#/accounts'}>
        {t('board.empty.gotoAccounts')}
      </button>
    </div>
  );
}
```

---

## 7. 模块依赖关系

```
                        ┌──────────────┐
                        │ Tauri App    │
                        │ (入口)       │
                        └──────┬───────┘
                               │ 创建 + 管理
            ┌──────────────────┼──────────────────┐
            ▼                  ▼                  ▼
     ┌─────────────┐   ┌─────────────┐   ┌─────────────┐
     │ Commands    │   │ Tray        │   │ Reminder    │
     │ (Controller)│   │ (Tauri API) │   │ Engine      │
     └──────┬──────┘   └──────┬──────┘   └──────┬──────┘
            │                 │ app_handle        │ app_handle
     ┌──────▼─────────────────▼──────────────────▼──────┐
     │                Service 层 (Rust)                 │
     │  MailFetcher  AIAnalyzer  AppScheduler           │
     └──────┬──────────┬──────────┬─────────────────────┘
            │          │          │
     ┌──────▼──┐  ┌────▼─────┐  ┌▼──────────┐
     │Sanitizer│  │SkillLoader│  │Settings   │
     └─────────┘  └──────────┘  │ Repo      │
                                └───────────┘
            │          │
     ┌──────▼──────────▼──────────────────────────────┐
     │              Repository 层 (Rust)               │
     │  ItemRepo  AccountRepo  SkillRepo  SkipListRepo │
     └──────────────────┬─────────────────────────────┘
                        │
                 ┌──────▼──────┐
                 │  Database   │
                 │ (rusqlite)  │
                 └──────┬──────┘
                        │
                 ┌──────▼──────┐
                 │ SecretStore │
                 │ (keyring)   │
                 └─────────────┘

  ┌───────────────────────────────────────────┐
  │ View 层 (WebView / React / TypeScript)    │
  │  通过 invoke() / listen() → Tauri IPC    │
  │  不直接引用 Rust 后端任何模块              │
  └───────────────────────────────────────────┘
```

**依赖规则：**
- **View → Tauri IPC → Commands**：前端通过 `invoke()` → Tauri IPC → Rust command 函数
- **Commands → Service/Repo**：command 函数调用 Service 方法或直接调用 Repo
- **Service → Repo**：Service 层协调多个 Repo，不直接操作数据库
- **Repo → Database**：每个 Repo 持有 Arc<Database> 引用
- **同层不互相依赖**：Repo 之间不互相调用，Service 之间不互相调用
- **app_handle 按需注入**：需要与前端交互的 Service 通过 `set_app_handle()` 获取引用

---

## 8. 功能 ↔ 模块映射表

| 功能 | 涉及的模块 |
|------|-----------|
| F1 邮箱管理 | `AccountRepo`, `SecretStore`, `account_commands`, `AccountsPage` |
| F2 邮件拉取 | `MailFetcher`, `AppScheduler`, `AccountRepo`, `SkipListRepo`, `Sanitizer`（截断） |
| F3 数据脱敏 | `Sanitizer` |
| F4 AI 分析 | `AIAnalyzer`, `Sanitizer`, `SkillLoader`, `SkillRepo`, `SettingsRepo`, `SecretStore` |
| F5 事项管理 | `ItemRepo` |
| F6 看板展示 | `BoardPage`, `ItemCard`, `ItemDetailDialog`, `ConfirmDialog` |
| F7 提醒通知 | `ReminderEngine`, `TrayManager`, `SettingsRepo`, `ItemRepo`（两段式到期提醒 + 过期边界处理 + 跨午夜免打扰） |
| F15 错误提示 | `AppScheduler`（ai:fatal-error 事件）, `BoardPage`（AI 异常提示条） | 新增（PRD P1-2） |
| F16 空状态引导 | `BoardEmptyState`, `AccountsPage`, `AISettingsPage` | 新增（PRD P2-8） |
| F17 事项自动清理 | `ItemRepo`（cleanup_old_completed，超过 1000 条时清理 >30 天的旧记录） | 新增（PRD P2-6） |
| F8 Skills 管理 | `SkillRepo`, `SkillLoader`, `SkillsPage` |
| F9 Skill 编辑器 | `SkillEditorPage`, `SkillTemplateEditor`, `SkillLoader` |
| F10 跳过名单 | `SkipListRepo`, `SkipListPage`, `ConfirmDialog` |
| F11 AI 设置 | `SettingsRepo`, `SecretStore`, `AISettingsPage` |
| F12 通用设置 | `SettingsRepo`, `GeneralPage` |
| F13 系统托盘 | `TrayManager`（Tauri Tray API + tauri-plugin-notification） |
| F14 i18n | `i18next`（外部库），`GeneralPage`（语言切换触发） |

---

## 9. Electron → Tauri 迁移对照表

> 方便理解两个版本的技术差异。

| 维度 | Electron 版 | Tauri 版 |
|------|------------|---------|
| 后端语言 | TypeScript (Node.js) | Rust |
| 前端语言 | TypeScript (React) | TypeScript (React)——**不变** |
| IPC 机制 | preload + ipcMain/ipcRenderer | `#[tauri::command]` + `invoke()` |
| IPC 通道 | 字符串常量（运行时校验） | 函数名（**编译期校验**） |
| SQLite | better-sqlite3（C++ 原生模块，ABI 坑） | rusqlite（**静态编译，无 ABI 问题**） |
| IMAP | imapflow (Node.js) | imap crate (Rust) |
| 邮件解析 | mailparser (Node.js) | mail-parser crate (Rust) |
| HTTP | Node.js fetch | reqwest (Rust) |
| 脱敏 | JS RegExp | Rust regex crate |
| 密码存储 | Electron safeStorage + JSON 文件 | keyring crate（**系统凭证管理器**） |
| 通知 | Electron Notification API | tauri-plugin-notification |
| 托盘 | Electron Tray | Tauri Tray API + tauri-plugin-notification |
| 开机启动 | app.setLoginItemSettings | tauri-plugin-autostart |
| 前端→后端 | window.api.xxx() | api.xxx()（封装 invoke） |
| 后端→前端推送 | mainWindow.webContents.send() | app_handle.emit() |
| 安装包体积 | ~150MB | **< 20MB** |
| 原生模块风险 | ❌ 有 ABI 兼容问题 | ✅ **不存在** |

---

*本文档基于 Jiuj-PRD.md v2026-04-14 优化生成，技术栈从 Electron 迁移至 Tauri v2。*
*v2026-04-14：与 PRD v2026-04-14 全面对齐：*
*- SecretStore: keyring → AES-256-GCM 加密存入 SQLite*
*- 移除免打扰（DNDPeriod / is_dnd_active / DND 设置）*
*- Sanitizer 截断策略简化为直接尾部截断*
*- Item/ExtractedItem 新增 attachments 字段*
*- SkillLoader 移除安全扫描（v1.1 再引入）*
*- AppScheduler rescan 移除去重逻辑*
*- 数据库 schema 新增 attachments 列*
*- 所有 command 的 SecretStore 签名更新*
*v2026-04-17：根据代码实现同步更新：*
*- ItemStatus 简化为 Pending/Overdue，增加 to_str/from_str 方法*
*- Item 结构体删除 attachments 字段，增加 time 字段，所有字段添加 serde rename*
*- ExtractedItem 删除 attachments，增加 time/visible 字段*
*- AIConfig 替换为 AIProfile（多配置管理），增加 AIProviderInfo/ModelInfo 结构体*
*- Account/Skill/SkipEntry/AddAccountRequest 等结构体添加 serde rename*
*- 增加 NewSkill/NewSkipEntry/AccountUpdate 请求结构体*
*- constants.rs 删除未使用的常量（APP_VERSION/SKILL_DIR/DEFAULT_LOCALE/DEFAULT_CLOSE_ACTION/LLM_MAX_RETRIES）*
*- AI 厂商配置从 &'static str 常量改为运行时动态构建*
*- 启动流程增加 AIProfilesRepo*
*- invoke handler 更新为 ai 模块的多配置命令*
*- Controller 层 ai_commands 更名为 ai*
*- Model 层 Repos 增加 AIProfilesRepo*
*v2026-04-13.1：根据 PRD 评审报告修复 P0/P1/P2 问题（overdue 状态、错误分类、body 截断、跨午夜 DND、空状态等）并重新对齐。*
