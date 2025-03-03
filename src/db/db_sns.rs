use std::path::{Path, PathBuf};
use rusqlite::{Connection, Result as SqliteResult, Row};
use serde::{Serialize, Deserialize};
use log::{info, warn, error};

use crate::wx_core::utils::{WxCoreError, WxCoreResult, wx_core_error};
use crate::db::db_base::DBHandler;

/// Sns database handler
pub struct SnsHandler {
    pub db: DBHandler,
}

impl SnsHandler {
    /// Create a new Sns database handler
    pub fn new(db_path: impl AsRef<Path>) -> WxCoreResult<Self> {
        wx_core_error(|| {
            let db = DBHandler::new(db_path)?;
            Ok(Self { db })
        })
    }
    
    /// Get moments list
    pub fn get_moments_list(&self, limit: usize, offset: usize) -> WxCoreResult<Vec<serde_json::Value>> {
        wx_core_error(|| {
            let sql = format!(
                "SELECT * FROM SnsInfo ORDER BY createTime DESC LIMIT {} OFFSET {}",
                limit, offset
            );
            
            self.db.execute_query(&sql, &[])
        })
    }
    
    /// Get moment by ID
    pub fn get_moment_by_id(&self, moment_id: i64) -> WxCoreResult<Option<serde_json::Value>> {
        wx_core_error(|| {
            let sql = "SELECT * FROM SnsInfo WHERE snsId = ?";
            self.db.execute_query_one(sql, &[&moment_id])
        })
    }
    
    /// Get moments by username
    pub fn get_moments_by_username(&self, username: &str, limit: usize, offset: usize) -> WxCoreResult<Vec<serde_json::Value>> {
        wx_core_error(|| {
            let sql = format!(
                "SELECT * FROM SnsInfo 
                WHERE userName = ? 
                ORDER BY createTime DESC 
                LIMIT {} OFFSET {}",
                limit, offset
            );
            
            self.db.execute_query(&sql, &[&username])
        })
    }
    
    /// Search moments
    pub fn search_moments(&self, keyword: &str, limit: usize, offset: usize) -> WxCoreResult<Vec<serde_json::Value>> {
        wx_core_error(|| {
            let sql = format!(
                "SELECT * FROM SnsInfo 
                WHERE content LIKE ? 
                ORDER BY createTime DESC 
                LIMIT {} OFFSET {}",
                limit, offset
            );
            
            let keyword = format!("%{}%", keyword);
            self.db.execute_query(&sql, &[&keyword])
        })
    }
    
    /// Get moment comments
    pub fn get_moment_comments(&self, moment_id: i64) -> WxCoreResult<Vec<serde_json::Value>> {
        wx_core_error(|| {
            let sql = "SELECT * FROM SnsComment WHERE snsId = ? ORDER BY createTime ASC";
            self.db.execute_query(sql, &[&moment_id])
        })
    }
    
    /// Get moment likes
    pub fn get_moment_likes(&self, moment_id: i64) -> WxCoreResult<Vec<serde_json::Value>> {
        wx_core_error(|| {
            let sql = "SELECT * FROM SnsLike WHERE snsId = ? ORDER BY createTime ASC";
            self.db.execute_query(sql, &[&moment_id])
        })
    }
    
    /// Get moments count
    pub fn get_moments_count(&self) -> WxCoreResult<i64> {
        wx_core_error(|| {
            let sql = "SELECT COUNT(*) as count FROM SnsInfo";
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
    
    /// Get moments count by username
    pub fn get_moments_count_by_username(&self, username: &str) -> WxCoreResult<i64> {
        wx_core_error(|| {
            let sql = "SELECT COUNT(*) as count FROM SnsInfo WHERE userName = ?";
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
