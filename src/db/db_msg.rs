use std::path::{Path, PathBuf};
use rusqlite::{Connection, Result as SqliteResult, Row};
use serde::{Serialize, Deserialize};
use log::{info, warn, error};

use crate::wx_core::utils::{WxCoreError, WxCoreResult, wx_core_error};
use crate::db::db_base::DBHandler;

/// MSG database handler
pub struct MsgHandler {
    pub db: DBHandler,
}

impl MsgHandler {
    /// Create a new MSG database handler
    pub fn new(db_path: impl AsRef<Path>) -> WxCoreResult<Self> {
        wx_core_error(|| {
            let db = DBHandler::new(db_path)?;
            Ok(Self { db })
        })
    }
    
    /// Get chat messages
    pub fn get_chat_messages(&self, chat_id: &str, limit: usize, offset: usize) -> WxCoreResult<Vec<serde_json::Value>> {
        wx_core_error(|| {
            let sql = format!(
                "SELECT * FROM message WHERE talker = ? ORDER BY createTime DESC LIMIT {} OFFSET {}",
                limit, offset
            );
            
            self.db.execute_query(&sql, &[&chat_id])
        })
    }
    
    /// Get chat list
    pub fn get_chat_list(&self, limit: usize, offset: usize) -> WxCoreResult<Vec<serde_json::Value>> {
        wx_core_error(|| {
            let sql = format!(
                "SELECT talker, COUNT(*) as message_count, MAX(createTime) as last_message_time 
                FROM message 
                GROUP BY talker 
                ORDER BY last_message_time DESC 
                LIMIT {} OFFSET {}",
                limit, offset
            );
            
            self.db.execute_query(&sql, &[])
        })
    }
    
    /// Search messages
    pub fn search_messages(&self, keyword: &str, limit: usize, offset: usize) -> WxCoreResult<Vec<serde_json::Value>> {
        wx_core_error(|| {
            let sql = format!(
                "SELECT * FROM message 
                WHERE content LIKE ? 
                ORDER BY createTime DESC 
                LIMIT {} OFFSET {}",
                limit, offset
            );
            
            let keyword = format!("%{}%", keyword);
            self.db.execute_query(&sql, &[&keyword])
        })
    }
    
    /// Get message by ID
    pub fn get_message_by_id(&self, message_id: i64) -> WxCoreResult<Option<serde_json::Value>> {
        wx_core_error(|| {
            let sql = "SELECT * FROM message WHERE msgId = ?";
            self.db.execute_query_one(sql, &[&message_id])
        })
    }
    
    /// Get message count
    pub fn get_message_count(&self) -> WxCoreResult<i64> {
        wx_core_error(|| {
            let sql = "SELECT COUNT(*) as count FROM message";
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
    
    /// Get chat count
    pub fn get_chat_count(&self) -> WxCoreResult<i64> {
        wx_core_error(|| {
            let sql = "SELECT COUNT(DISTINCT talker) as count FROM message";
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
