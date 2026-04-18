use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(db_path)?;
        let db = Database {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init()?;
        db.migrate()?;
        Ok(db)
    }

    fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS items (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                deadline TEXT,
                time TEXT,
                priority TEXT NOT NULL,
                item_type TEXT NOT NULL,
                remind_offsets TEXT,
                notified_stages TEXT NOT NULL,
                source_email_id TEXT NOT NULL,
                source_from TEXT NOT NULL,
                source_subject TEXT NOT NULL,
                source_date TEXT NOT NULL,
                source_account TEXT NOT NULL,
                matched_skill TEXT,
                status TEXT NOT NULL,
                last_notified_at TEXT,
                created_at TEXT NOT NULL,
                completed_at TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                email TEXT NOT NULL,
                imap_host TEXT NOT NULL,
                imap_port INTEGER NOT NULL,
                last_uid INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS skills (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 0,
                sort_order INTEGER NOT NULL,
                is_builtin INTEGER NOT NULL DEFAULT 0,
                file_path TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS skip_list (
                id TEXT PRIMARY KEY,
                type TEXT NOT NULL,
                value TEXT NOT NULL,
                UNIQUE(type, value)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS ai_profiles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                base_url TEXT,
                custom_name TEXT,
                is_active INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    fn migrate(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();

        let has_time_column: bool = conn
            .prepare("SELECT time FROM items LIMIT 0")
            .is_ok();

        if !has_time_column {
            log::info!("[DB] Adding 'time' column to items table");
            conn.execute("ALTER TABLE items ADD COLUMN time TEXT", [])?;
        }

        let has_attachments_column: bool = conn
            .prepare("SELECT attachments FROM items LIMIT 0")
            .is_ok();

        if has_attachments_column {
            log::info!("[DB] Migrating items table: removing 'attachments' column");
            conn.execute_batch(
                "BEGIN TRANSACTION;
                 CREATE TABLE items_new (
                     id TEXT PRIMARY KEY,
                     content TEXT NOT NULL,
                     deadline TEXT,
                     time TEXT,
                     priority TEXT NOT NULL,
                     item_type TEXT NOT NULL,
                     remind_offsets TEXT,
                     notified_stages TEXT NOT NULL,
                     source_email_id TEXT NOT NULL,
                     source_from TEXT NOT NULL,
                     source_subject TEXT NOT NULL,
                     source_date TEXT NOT NULL DEFAULT '',
                     source_account TEXT NOT NULL,
                     matched_skill TEXT,
                     status TEXT NOT NULL,
                     last_notified_at TEXT,
                     created_at TEXT NOT NULL,
                     completed_at TEXT
                 );
                 INSERT INTO items_new
                     SELECT id, content, deadline, time, priority, item_type,
                            remind_offsets, notified_stages, source_email_id,
                            source_from, source_subject, '', source_account,
                            matched_skill, status, last_notified_at,
                            created_at, completed_at
                     FROM items;
                 DROP TABLE items;
                 ALTER TABLE items_new RENAME TO items;
                 COMMIT;"
            )?;
            log::info!("[DB] Migration complete: 'attachments' column removed");
        }

        let skip_list_has_unique: bool = conn
            .prepare("SELECT 1 FROM pragma_index_list('skip_list') WHERE name LIKE '%unique%' LIMIT 1")
            .map(|mut stmt| stmt.exists([]).unwrap_or(false))
            .unwrap_or(false);

        if !skip_list_has_unique {
            log::info!("[DB] Adding UNIQUE constraint to skip_list (type, value)");
            conn.execute_batch(
                "BEGIN TRANSACTION;
                 CREATE TABLE skip_list_new (
                     id TEXT PRIMARY KEY,
                     type TEXT NOT NULL,
                     value TEXT NOT NULL,
                     UNIQUE(type, value)
                 );
                 INSERT OR IGNORE INTO skip_list_new SELECT id, type, value FROM skip_list;
                 DROP TABLE skip_list;
                 ALTER TABLE skip_list_new RENAME TO skip_list;
                 COMMIT;"
            )?;
            log::info!("[DB] Migration complete: skip_list UNIQUE constraint added");
        }

        let has_source_date: bool = conn
            .prepare("SELECT source_date FROM items LIMIT 0")
            .is_ok();

        if !has_source_date {
            log::info!("[DB] Adding 'source_date' column to items table");
            conn.execute("ALTER TABLE items ADD COLUMN source_date TEXT NOT NULL DEFAULT ''", [])?;
        }

        Ok(())
    }

    pub fn get_connection(&self) -> Arc<Mutex<Connection>> {
        self.conn.clone()
    }
}
