use std::path::Path;

use crate::db::db_base::DBHandler;
use crate::wx_core::utils::{wx_core_error, WxCoreResult};

/// OpenIMContact database handler
pub struct OpenIMContactHandler {
    pub db: DBHandler,
}

impl OpenIMContactHandler {
    /// Create a new OpenIMContact database handler
    pub fn new(db_path: impl AsRef<Path>) -> WxCoreResult<Self> {
        wx_core_error(|| {
            let db = DBHandler::new(db_path)?;
            Ok(Self { db })
        })
    }

    /// Get contact list
    pub fn get_contact_list(
        &self,
        limit: usize,
        offset: usize,
    ) -> WxCoreResult<Vec<serde_json::Value>> {
        wx_core_error(|| {
            let sql = format!(
                "SELECT * FROM OpenIMContact ORDER BY UserName LIMIT {} OFFSET {}",
                limit, offset
            );

            self.db.execute_query(&sql, &[])
        })
    }

    /// Get contact by username
    pub fn get_contact_by_username(
        &self,
        username: &str,
    ) -> WxCoreResult<Option<serde_json::Value>> {
        wx_core_error(|| {
            let sql = "SELECT * FROM OpenIMContact WHERE UserName = ?";
            self.db.execute_query_one(sql, &[&username])
        })
    }

    /// Search contacts
    pub fn search_contacts(
        &self,
        keyword: &str,
        limit: usize,
        offset: usize,
    ) -> WxCoreResult<Vec<serde_json::Value>> {
        wx_core_error(|| {
            let sql = format!(
                "SELECT * FROM OpenIMContact 
                WHERE UserName LIKE ? OR NickName LIKE ? 
                ORDER BY UserName 
                LIMIT {} OFFSET {}",
                limit, offset
            );

            let keyword = format!("%{}%", keyword);
            self.db.execute_query(&sql, &[&keyword, &keyword])
        })
    }

    /// Get contact count
    pub fn get_contact_count(&self) -> WxCoreResult<i64> {
        wx_core_error(|| {
            let sql = "SELECT COUNT(*) as count FROM OpenIMContact";
            let result = self.db.execute_query_one(sql, &[])?;

            if let Some(serde_json::Value::Object(map)) = result {
                if let Some(serde_json::Value::Number(count)) = map.get("count") {
                    if let Some(count) = count.as_i64() {
                        return Ok(count);
                    }
                }
            }

            Ok(0)
        })
    }

    /// Close the database connection
    pub fn close(self) -> WxCoreResult<()> {
        self.db.close()
    }
}
