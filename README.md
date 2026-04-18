# Jiuj (啾啾)

有邮件来了，该做的事别忘了。

## 项目简介

Jiuj 是一个轻量级 Tauri v2 桌面工具，通过 IMAP 连接邮箱，使用 AI 自动提取重要信息（任务、日程、截止日期等），并在合适时间提醒您。

## 技术栈

- **前端**：React + TypeScript + Vite + Tailwind CSS
- **后端**：Rust + Tauri v2
- **数据库**：SQLite
- **AI**：支持 OpenAI/DeepSeek/Kimi/智谱/通义千问/Claude/Ollama

## 项目结构

```
Jiuj/
├── src/                    # 前端 (React/TypeScript)
│   ├── pages/             # 页面组件
│   ├── i18n/              # 国际化
│   ├── types.ts           # 类型定义
│   └── tauri-api.ts       # Tauri API 封装
├── src-tauri/             # 后端 (Rust)
│   ├── src/
│   │   ├── db/            # 数据库层
│   │   ├── services/      # 服务层
│   │   ├── skills/        # Skill 加载
│   │   ├── commands/      # Tauri 命令
│   │   └── main.rs        # 入口
│   └── Cargo.toml         # Rust 依赖
└── default/               # 内置 Skill
    └── SKILL.md
```

## 快速开始

### 前置要求

- Node.js 18+
- Rust (通过 rustup 安装)

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri:dev
```

### 构建

```bash
npm run tauri:build
```

## 功能特性

- ✅ 邮箱 IMAP 只读连接
- ✅ 智能邮件预处理（脱敏 + 截断）
- ✅ 多厂商 AI 分析
- ✅ 事项看板（进行中/已完成）
- ✅ 两段式到期提醒
- ✅ Skills 自定义
- ✅ 跳过名单（发件人/域名）
- ✅ 国际化（中文/英文）
- ✅ 托盘菜单

## License

MIT
