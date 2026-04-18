use crate::models::*;
use crate::db::Database;
use std::sync::Arc;
use uuid::Uuid;
use rusqlite::params;

#[derive(Clone)]
pub struct AccountRepo {
    pub db: Arc<Database>,
}

impl AccountRepo {
    pub fn new(db: Arc<Database>) -> Self {
        AccountRepo { db }
    }

    pub fn insert(&self, email: &str, imap_host: &str, imap_port: u16) -> Result<Account, Box<dyn std::error::Error>> {
        // Check if email already exists
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM accounts WHERE email = ?1")?;
        let count: i64 = stmt.query_row([email], |row| row.get(0))?;
        
        if count > 0 {
            return Err("该邮箱地址已存在".into());
        }
        
        let id = Uuid::new_v4().to_string();
        let account = Account {
            id: id.clone(),
            email: email.to_string(),
            imap_host: imap_host.to_string(),
            imap_port,
            last_uid: 0,
            status: AccountStatus::Active,
        };

        conn.execute(
            "INSERT INTO accounts (id, email, imap_host, imap_port, last_uid, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                account.id,
                account.email,
                account.imap_host,
                account.imap_port,
                account.last_uid,
                "active",
            ],
        )?;

        Ok(account)
    }

    pub fn get_all(&self) -> Result<Vec<Account>, Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM accounts")?;
        let accounts_iter = stmt.query_map([], |row| self.row_to_account(row))?;
        let mut accounts = Vec::new();
        for account_result in accounts_iter {
            accounts.push(account_result?);
        }
        Ok(accounts)
    }

    pub fn update(&self, id: &str, data: AccountUpdate) -> Result<(), Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        
        if let Some(email) = data.email {
            conn.execute("UPDATE accounts SET email = ?1 WHERE id = ?2", params![email, id])?;
        }
        if let Some(imap_host) = data.imap_host {
            conn.execute("UPDATE accounts SET imap_host = ?1 WHERE id = ?2", params![imap_host, id])?;
        }
        if let Some(imap_port) = data.imap_port {
            conn.execute("UPDATE accounts SET imap_port = ?1 WHERE id = ?2", params![imap_port, id])?;
        }

        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        conn.execute("DELETE FROM accounts WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_last_uid(&self, id: &str, uid: u64) -> Result<(), Box<dyn std::error::Error>> {
        let conn_arc = self.db.get_connection();
        let conn = conn_arc.lock().unwrap();
        conn.execute("UPDATE accounts SET last_uid = ?1 WHERE id = ?2", params![uid, id])?;
        Ok(())
    }

    fn row_to_account(&self, row: &rusqlite::Row) -> Result<Account, rusqlite::Error> {
        let status_str: String = row.get(5)?;
        let status = if status_str == "active" { AccountStatus::Active } else { AccountStatus::Disabled };
        Ok(Account {
            id: row.get(0)?,
            email: row.get(1)?,
            imap_host: row.get(2)?,
            imap_port: row.get(3)?,
            last_uid: row.get(4)?,
            status,
        })
    }
}
