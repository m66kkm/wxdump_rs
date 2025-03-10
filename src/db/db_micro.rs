use std::path::Path;

use crate::db::db_base::DBHandler;
use crate::wx_core::utils::{wx_core_error, WxCoreResult};

/// MicroMsg database handler
pub struct MicroHandler {
    pub db: DBHandler,
}

impl MicroHandler {
    /// Create a new MicroMsg database handler
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
                "SELECT * FROM contact ORDER BY username LIMIT {} OFFSET {}",
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
            let sql = "SELECT * FROM contact WHERE username = ?";
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
                "SELECT * FROM contact 
                WHERE username LIKE ? OR nickname LIKE ? 
                ORDER BY username 
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
            let sql = "SELECT COUNT(*) as count FROM contact";
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

    /// Get chat room members
    pub fn get_chatroom_members(&self, chatroom_id: &str) -> WxCoreResult<Vec<serde_json::Value>> {
        wx_core_error(|| {
            let sql = "SELECT * FROM chatroom WHERE chatroomname = ?";
            let result = self.db.execute_query_one(sql, &[&chatroom_id])?;

            if let Some(serde_json::Value::Object(map)) = result {
                if let Some(serde_json::Value::String(members)) = map.get("memberlist") {
                    // Parse the member list
                    // In the actual implementation, this would involve parsing the member list format
                    // For now, we'll just return a placeholder
                    let members_vec = members
                        .split(';')
                        .map(|m| {
                            serde_json::json!({
                                "username": m.trim(),
                            })
                        })
                        .collect();

                    return Ok(members_vec);
                }
            }

            Ok(Vec::new())
        })
    }

    /// Close the database connection
    pub fn close(self) -> WxCoreResult<()> {
        self.db.close()
    }
}
