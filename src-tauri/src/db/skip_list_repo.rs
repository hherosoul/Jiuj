use crate::models::*;
use crate::db::Database;
use std::sync::Arc;
use uuid::Uuid;
use rusqlite::params;
use serde_json;

#[derive(Clone)]
pub struct SkipListRepo {
    db: Arc<Database>,
}

impl SkipListRepo {
    pub fn new(db: Arc<Database>) -> Self {
        SkipListRepo { db }
    }

    pub fn insert(&self, entry: NewSkipEntry) -> Result<SkipEntry, Box<dyn std::error::Error>> {
        let id = Uuid::new_v4().to_string();
        let skip_entry = SkipEntry {
            id: id.clone(),
            skip_type: entry.skip_type,
            value: entry.value,
        };

        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        conn.execute(
            "INSERT INTO skip_list (id, type, value) VALUES (?1, ?2, ?3)",
            params![
                skip_entry.id,
                serde_json::to_string(&skip_entry.skip_type)?,
                skip_entry.value,
            ],
        )?;

        Ok(skip_entry)
    }

    pub fn get_all(&self) -> Result<Vec<SkipEntry>, Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM skip_list")?;
        let entries_iter = stmt.query_map([], |row| self.row_to_skip_entry(row))?;
        let mut entries = Vec::new();
        for entry_result in entries_iter {
            entries.push(entry_result?);
        }
        Ok(entries)
    }

    pub fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        conn.execute("DELETE FROM skip_list WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn is_sender_skipped(&self, email: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM skip_list WHERE type = 'sender' AND value = ?1",
            params![email],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn is_domain_skipped(&self, domain: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM skip_list WHERE type = 'domain' AND value = ?1",
            params![domain],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn is_skipped(&self, from: &str) -> Result<bool, Box<dyn std::error::Error>> {
        if self.is_sender_skipped(from)? {
            return Ok(true);
        }

        if let Some(domain) = from.split('@').nth(1) {
            if self.is_domain_skipped(domain)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn row_to_skip_entry(&self, row: &rusqlite::Row) -> Result<SkipEntry, rusqlite::Error> {
        let skip_type_str: String = row.get(1)?;
        Ok(SkipEntry {
            id: row.get(0)?,
            skip_type: serde_json::from_str(&skip_type_str).unwrap_or(SkipType::Sender),
            value: row.get(2)?,
        })
    }
}
