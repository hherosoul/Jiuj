use crate::models::*;
use crate::db::Database;
use std::sync::Arc;
use uuid::Uuid;
use rusqlite::params;
use chrono::Utc;

#[derive(Clone)]
pub struct AIProfilesRepo {
    pub db: Arc<Database>,
}

impl AIProfilesRepo {
    pub fn new(db: Arc<Database>) -> Self {
        AIProfilesRepo { db }
    }

    pub fn insert(
        &self,
        name: &str,
        provider: AIProvider,
        model: &str,
        base_url: Option<&str>,
        custom_name: Option<&str>,
    ) -> Result<AIProfile, Box<dyn std::error::Error>> {
        let id = Uuid::new_v4().to_string();
        let provider_str = match provider {
            AIProvider::OpenAI => "openai",
            AIProvider::DeepSeek => "deepseek",
            AIProvider::Kimi => "kimi",
            AIProvider::Zhipu => "zhipu",
            AIProvider::Qwen => "qwen",
            AIProvider::Claude => "claude",
            AIProvider::Ollama => "ollama",
            AIProvider::Custom => "custom",
        };
        let created_at = Utc::now().to_rfc3339();
        
        let all_profiles = self.get_all()?;
        let is_active = all_profiles.is_empty();

        let profile = AIProfile {
            id: id.clone(),
            name: name.to_string(),
            provider,
            model: model.to_string(),
            base_url: base_url.map(|s| s.to_string()),
            custom_name: custom_name.map(|s| s.to_string()),
            is_active,
            created_at,
        };

        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        conn.execute(
            "INSERT INTO ai_profiles (id, name, provider, model, base_url, custom_name, is_active, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                profile.id,
                profile.name,
                provider_str,
                profile.model,
                profile.base_url,
                profile.custom_name,
                if profile.is_active { 1 } else { 0 },
                profile.created_at,
            ],
        )?;

        Ok(profile)
    }

    pub fn get_all(&self) -> Result<Vec<AIProfile>, Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM ai_profiles ORDER BY is_active DESC, created_at DESC")?;
        let profiles_iter = stmt.query_map([], |row| self.row_to_profile(row))?;
        let mut profiles = Vec::new();
        for profile_result in profiles_iter {
            profiles.push(profile_result?);
        }
        Ok(profiles)
    }

    pub fn get_active(&self) -> Result<Option<AIProfile>, Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM ai_profiles WHERE is_active = 1 LIMIT 1")?;
        let mut profiles_iter = stmt.query_map([], |row| self.row_to_profile(row))?;
        
        if let Some(profile_result) = profiles_iter.next() {
            Ok(Some(profile_result?))
        } else {
            Ok(None)
        }
    }

    pub fn set_active(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        
        conn.execute("UPDATE ai_profiles SET is_active = 0 WHERE is_active = 1", [])?;
        conn.execute("UPDATE ai_profiles SET is_active = 1 WHERE id = ?1", params![id])?;
        
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        conn.execute("DELETE FROM ai_profiles WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update(
        &self,
        id: &str,
        name: Option<&str>,
        model: Option<&str>,
        base_url: Option<Option<String>>,
        custom_name: Option<Option<String>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        
        if let Some(n) = name {
            conn.execute("UPDATE ai_profiles SET name = ?1 WHERE id = ?2", params![n, id])?;
        }
        if let Some(m) = model {
            conn.execute("UPDATE ai_profiles SET model = ?1 WHERE id = ?2", params![m, id])?;
        }
        if let Some(bu) = base_url {
            conn.execute("UPDATE ai_profiles SET base_url = ?1 WHERE id = ?2", params![bu, id])?;
        }
        if let Some(cn) = custom_name {
            conn.execute("UPDATE ai_profiles SET custom_name = ?1 WHERE id = ?2", params![cn, id])?;
        }

        Ok(())
    }

    fn row_to_profile(&self, row: &rusqlite::Row) -> Result<AIProfile, rusqlite::Error> {
        let provider_str: String = row.get(2)?;
        let provider = match provider_str.as_str() {
            "openai" => AIProvider::OpenAI,
            "deepseek" => AIProvider::DeepSeek,
            "kimi" => AIProvider::Kimi,
            "zhipu" => AIProvider::Zhipu,
            "qwen" => AIProvider::Qwen,
            "claude" => AIProvider::Claude,
            "ollama" => AIProvider::Ollama,
            "custom" => AIProvider::Custom,
            _ => AIProvider::OpenAI,
        };
        
        let is_active_int: i32 = row.get(6)?;
        
        Ok(AIProfile {
            id: row.get(0)?,
            name: row.get(1)?,
            provider,
            model: row.get(3)?,
            base_url: row.get(4)?,
            custom_name: row.get(5)?,
            is_active: is_active_int != 0,
            created_at: row.get(7)?,
        })
    }
}
