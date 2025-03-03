use std::path::Path;

use crate::wx_core::utils::{WxCoreResult, wx_core_error};
use crate::db::db_base::DBHandler;

/// PublicMsg database handler
pub struct PublicMsgHandler {
    pub db: DBHandler,
}

impl PublicMsgHandler {
    /// Create a new PublicMsg database handler
    pub fn new(db_path: impl AsRef<Path>) -> WxCoreResult<Self> {
        wx_core_error(|| {
            let db = DBHandler::new(db_path)?;
            Ok(Self { db })
        })
    }
    
    /// Get public message list
    pub fn get_public_msg_list(&self, limit: usize, offset: usize) -> WxCoreResult<Vec<serde_json::Value>> {
        wx_core_error(|| {
            let sql = format!(
                "SELECT * FROM PublicMsg ORDER BY MsgId DESC LIMIT {} OFFSET {}",
                limit, offset
            );
            
            self.db.execute_query(&sql, &[])
        })
    }
    
    /// Get public message by ID
    pub fn get_public_msg_by_id(&self, msg_id: i64) -> WxCoreResult<Option<serde_json::Value>> {
        wx_core_error(|| {
            let sql = "SELECT * FROM PublicMsg WHERE MsgId = ?";
            self.db.execute_query_one(sql, &[&msg_id])
        })
    }
    
    /// Get public message by username
    pub fn get_public_msg_by_username(&self, username: &str, limit: usize, offset: usize) -> WxCoreResult<Vec<serde_json::Value>> {
        wx_core_error(|| {
            let sql = format!(
                "SELECT * FROM PublicMsg 
                WHERE UserName = ? 
                ORDER BY MsgId DESC 
                LIMIT {} OFFSET {}",
                limit, offset
            );
            
            self.db.execute_query(&sql, &[&username])
        })
    }
    
    /// Search public messages
    pub fn search_public_msg(&self, keyword: &str, limit: usize, offset: usize) -> WxCoreResult<Vec<serde_json::Value>> {
        wx_core_error(|| {
            let sql = format!(
                "SELECT * FROM PublicMsg 
                WHERE Content LIKE ? 
                ORDER BY MsgId DESC 
                LIMIT {} OFFSET {}",
                limit, offset
            );
            
            let keyword = format!("%{}%", keyword);
            self.db.execute_query(&sql, &[&keyword])
        })
    }
    
    /// Get public message count
    pub fn get_public_msg_count(&self) -> WxCoreResult<i64> {
        wx_core_error(|| {
            let sql = "SELECT COUNT(*) as count FROM PublicMsg";
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
    
    /// Get public message count by username
    pub fn get_public_msg_count_by_username(&self, username: &str) -> WxCoreResult<i64> {
        wx_core_error(|| {
            let sql = "SELECT COUNT(*) as count FROM PublicMsg WHERE UserName = ?";
            let result = self.db.execute_query_one(sql, &[&username])?;
            
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
