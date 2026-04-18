use crate::db::Database;
use std::sync::Arc;
use rusqlite::params;
use std::collections::HashMap;

#[derive(Clone)]
pub struct SettingsRepo {
    db: Arc<Database>,
}

impl SettingsRepo {
    pub fn new(db: Arc<Database>) -> Self {
        SettingsRepo { db }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        ).ok()
    }

    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }

    pub fn set(&self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_all(&self, keys: Option<Vec<String>>) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        let mut map = HashMap::new();

        if let Some(keys) = keys {
            for key in keys {
                if let Some(value) = self.get(&key) {
                    map.insert(key, value);
                }
            }
        } else {
            let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
            for row in rows {
                let (k, v) = row?;
                map.insert(k, v);
            }
        }

        Ok(map)
    }
}
