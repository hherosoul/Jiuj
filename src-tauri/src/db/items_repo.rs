use crate::models::*;
use crate::db::Database;
use chrono::Utc;
use rusqlite::params;
use std::sync::Arc;
use serde_json;
use uuid::Uuid;

#[derive(Clone)]
pub struct ItemRepo {
    db: Arc<Database>,
}

impl ItemRepo {
    pub fn new(db: Arc<Database>) -> Self {
        ItemRepo { db }
    }

    pub fn insert(&self, extracted_item: &ExtractedItem, source_email_id: &str, source_from: &str, source_subject: &str, source_date: &str, source_account: &str, matched_skill: Option<&str>) -> Result<Item, Box<dyn std::error::Error>> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let notified_stages = Vec::<usize>::new();
        let item = Item {
            id: id.clone(),
            content: extracted_item.content.clone(),
            deadline: extracted_item.deadline.clone(),
            time: extracted_item.time.clone(),
            priority: extracted_item.priority.clone(),
            item_type: extracted_item.item_type.clone(),
            remind_offsets: extracted_item.remind_offsets.clone(),
            notified_stages,
            source_email_id: source_email_id.to_string(),
            source_from: source_from.to_string(),
            source_subject: source_subject.to_string(),
            source_date: source_date.to_string(),
            source_account: source_account.to_string(),
            matched_skill: matched_skill.map(|s| s.to_string()),
            status: ItemStatus::Pending,
            last_notified_at: None,
            created_at: now.clone(),
            completed_at: None,
        };

        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        conn.execute(
            "INSERT INTO items (id, content, deadline, time, priority, item_type, remind_offsets, notified_stages, source_email_id, source_from, source_subject, source_date, source_account, matched_skill, status, last_notified_at, created_at, completed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            params![
                item.id,
                item.content,
                item.deadline,
                item.time,
                serde_json::to_string(&item.priority)?,
                serde_json::to_string(&item.item_type)?,
                item.remind_offsets.as_ref().map(|o| serde_json::to_string(o).unwrap()),
                serde_json::to_string(&item.notified_stages)?,
                item.source_email_id,
                item.source_from,
                item.source_subject,
                item.source_date,
                item.source_account,
                item.matched_skill,
                item.status.to_str(),
                item.last_notified_at,
                item.created_at,
                item.completed_at,
            ],
        )?;

        Ok(item)
    }

    const SELECT_COLUMNS: &str = "id, content, deadline, time, priority, item_type, remind_offsets, notified_stages, source_email_id, source_from, source_subject, source_date, source_account, matched_skill, status, last_notified_at, created_at, completed_at";

    pub fn get_all(&self) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        let mut stmt = conn.prepare(&format!("SELECT {} FROM items ORDER BY created_at DESC", Self::SELECT_COLUMNS))?;
        let items_iter = stmt.query_map([], |row| Self::row_to_item(row))?;
        let mut items = Vec::new();
        for item_result in items_iter {
            items.push(item_result?);
        }
        Ok(items)
    }

    pub fn get_pending(&self) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        let mut stmt = conn.prepare(&format!("SELECT {} FROM items WHERE status = 'pending' OR status = 'overdue' ORDER BY CASE status WHEN 'overdue' THEN 0 ELSE 1 END, deadline ASC NULLS LAST, created_at DESC", Self::SELECT_COLUMNS))?;
        let items_iter = stmt.query_map([], |row| Self::row_to_item(row))?;
        let mut items = Vec::new();
        for item_result in items_iter {
            items.push(item_result?);
        }
        Ok(items)
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<Item>, Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        let mut stmt = conn.prepare(&format!("SELECT {} FROM items WHERE id = ?1", Self::SELECT_COLUMNS))?;
        let mut items_iter = stmt.query_map([id], |row| Self::row_to_item(row))?;
        Ok(items_iter.next().transpose()?)
    }

    pub fn complete(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        conn.execute("DELETE FROM items WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn ignore(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        conn.execute("DELETE FROM items WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_notified_stages(&self, id: &str, stages: &[usize]) -> Result<(), Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        conn.execute(
            "UPDATE items SET notified_stages = ?1, last_notified_at = ?2 WHERE id = ?3",
            params![serde_json::to_string(stages)?, Utc::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    pub fn mark_overdue(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE items SET status = 'overdue' WHERE status = 'pending' AND deadline IS NOT NULL AND deadline < ?1",
            params![now],
        )?;
        Ok(())
    }

    fn row_to_item(row: &rusqlite::Row) -> Result<Item, rusqlite::Error> {
        let priority_str: String = row.get("priority")?;
        let item_type_str: String = row.get("item_type")?;
        let remind_offsets_str: Option<String> = row.get("remind_offsets")?;
        let notified_stages_str: String = row.get("notified_stages")?;
        let status_str: String = row.get("status")?;

        Ok(Item {
            id: row.get("id")?,
            content: row.get("content")?,
            deadline: row.get("deadline")?,
            time: row.get("time")?,
            priority: serde_json::from_str(&priority_str).unwrap_or(Priority::Medium),
            item_type: serde_json::from_str(&item_type_str).unwrap_or(ItemType::Other),
            remind_offsets: remind_offsets_str.map(|s| serde_json::from_str(&s).unwrap_or_default()),
            notified_stages: serde_json::from_str(&notified_stages_str).unwrap_or_default(),
            source_email_id: row.get("source_email_id")?,
            source_from: row.get("source_from")?,
            source_subject: row.get("source_subject")?,
            source_date: row.get("source_date")?,
            source_account: row.get("source_account")?,
            matched_skill: row.get("matched_skill")?,
            status: ItemStatus::from_str(&status_str),
            last_notified_at: row.get("last_notified_at")?,
            created_at: row.get("created_at")?,
            completed_at: row.get("completed_at")?,
        })
    }
}
