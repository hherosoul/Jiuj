# Jiuj (啾啾)

Don't forget what needs to be done when emails arrive.

## Project Overview

Jiuj is a lightweight Tauri v2 desktop tool that connects to your email via IMAP, uses AI to automatically extract important information (tasks, schedules, deadlines, etc.), and reminds you at the right time.

## Tech Stack

- **Frontend**: React + TypeScript + Vite + Tailwind CSS
- **Backend**: Rust + Tauri v2
- **Database**: SQLite
- **AI**: OpenAI/DeepSeek/Kimi/Zhipu/Qwen/Claude/Ollama

## Project Structure

```
Jiuj/
├── src/                    # Frontend (React/TypeScript)
│   ├── pages/             # Page components
│   ├── i18n/              # Internationalization
│   ├── types.ts           # Type definitions
│   └── tauri-api.ts       # Tauri API wrapper
├── src-tauri/             # Backend (Rust)
│   ├── src/
│   │   ├── db/            # Database layer
│   │   ├── services/      # Service layer
│   │   ├── skills/        # Skill loader
│   │   ├── commands/      # Tauri commands
│   │   └── main.rs        # Entry point
│   └── Cargo.toml         # Rust dependencies
└── default/               # Built-in Skill
    └── SKILL.md
```

## Quick Start

### Prerequisites

- Node.js 18+
- Rust (install via rustup)

### Install Dependencies

```bash
npm install
```

### Development Mode

```bash
npm run tauri:dev
```

### Build

```bash
npm run tauri:build
```

## Features

- ✅ Email IMAP read-only connection
- ✅ Smart email preprocessing (sanitization + truncation)
- ✅ Multi-provider AI analysis
- ✅ Task board (In Progress/Completed)
- ✅ Two-stage deadline reminders
- ✅ Custom Skills
- ✅ Skip list (sender/domain)
- ✅ Internationalization (Chinese/English)
- ✅ Tray menu

## Download

### macOS

Download: [Jiuj_0.1.0_aarch64.dmg](https://github.com/hherosoul/Jiuj/releases/download/v0.1.0/Jiuj_0.1.0_aarch64.dmg)

### Windows

Windows version coming soon.

## License

MIT
