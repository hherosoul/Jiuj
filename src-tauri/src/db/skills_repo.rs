use crate::models::*;
use crate::db::Database;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;
use rusqlite::params;

#[derive(Clone)]
pub struct SkillRepo {
    db: Arc<Database>,
}

impl SkillRepo {
    pub fn new(db: Arc<Database>) -> Self {
        SkillRepo { db }
    }

    pub fn insert(&self, skill: NewSkill) -> Result<Skill, Box<dyn std::error::Error>> {
        // Check if skill name already exists
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM skills WHERE name = ?1")?;
        let count: i64 = stmt.query_row([&skill.name], |row| row.get(0))?;
        
        if count > 0 {
            return Err("该技能名称已存在".into());
        }
        
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let skill = Skill {
            id: id.clone(),
            name: skill.name,
            description: skill.description,
            enabled: false,
            sort_order: skill.sort_order,
            is_builtin: skill.is_builtin,
            file_path: skill.file_path,
            updated_at: now.clone(),
        };

        conn.execute(
            "INSERT INTO skills (id, name, description, enabled, sort_order, is_builtin, file_path, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                skill.id,
                skill.name,
                skill.description,
                if skill.enabled { 1 } else { 0 },
                skill.sort_order,
                if skill.is_builtin { 1 } else { 0 },
                skill.file_path,
                skill.updated_at,
            ],
        )?;

        Ok(skill)
    }

    pub fn get_all(&self) -> Result<Vec<Skill>, Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM skills ORDER BY sort_order ASC")?;
        let skills_iter = stmt.query_map([], |row| self.row_to_skill(row))?;
        let mut skills = Vec::new();
        for skill_result in skills_iter {
            skills.push(skill_result?);
        }
        Ok(skills)
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<Skill>, Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM skills WHERE id = ?1")?;
        let mut skills_iter = stmt.query_map([id], |row| self.row_to_skill(row))?;
        Ok(skills_iter.next().transpose()?)
    }

    pub fn get_by_name(&self, name: &str) -> Result<Option<Skill>, Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM skills WHERE name = ?1")?;
        let mut skills_iter = stmt.query_map([name], |row| self.row_to_skill(row))?;
        Ok(skills_iter.next().transpose()?)
    }

    pub fn get_active(&self) -> Result<Option<Skill>, Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM skills WHERE enabled = 1 LIMIT 1")?;
        let mut skills_iter = stmt.query_map([], |row| self.row_to_skill(row))?;
        Ok(skills_iter.next().transpose()?)
    }

    pub fn set_active(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        conn.execute("UPDATE skills SET enabled = 0", [])?;
        conn.execute("UPDATE skills SET enabled = 1 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn touch(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE skills SET updated_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        
        let is_builtin: bool = conn.query_row(
            "SELECT is_builtin FROM skills WHERE id = ?1",
            params![id],
            |row| Ok(row.get::<_, i32>(0)? == 1),
        ).unwrap_or(false);

        if is_builtin {
            return Err("Cannot delete built-in skill".into());
        }

        conn.execute("DELETE FROM skills WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn row_to_skill(&self, row: &rusqlite::Row) -> Result<Skill, rusqlite::Error> {
        Ok(Skill {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            enabled: row.get::<_, i32>(3)? == 1,
            sort_order: row.get(4)?,
            is_builtin: row.get::<_, i32>(5)? == 1,
            file_path: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }
}
