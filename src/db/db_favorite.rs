use std::path::Path;

use crate::db::db_base::DBHandler;
use crate::wx_core::utils::{wx_core_error, WxCoreResult};

/// Favorite database handler
pub struct FavoriteHandler {
    pub db: DBHandler,
}

impl FavoriteHandler {
    /// Create a new Favorite database handler
    pub fn new(db_path: impl AsRef<Path>) -> WxCoreResult<Self> {
        wx_core_error(|| {
            let db = DBHandler::new(db_path)?;
            Ok(Self { db })
        })
    }

    /// Get favorite list
    pub fn get_favorite_list(
        &self,
        limit: usize,
        offset: usize,
    ) -> WxCoreResult<Vec<serde_json::Value>> {
        wx_core_error(|| {
            let sql = format!(
                "SELECT * FROM FavItem ORDER BY localId DESC LIMIT {} OFFSET {}",
                limit, offset
            );

            self.db.execute_query(&sql, &[])
        })
    }

    /// Get favorite by ID
    pub fn get_favorite_by_id(&self, favorite_id: i64) -> WxCoreResult<Option<serde_json::Value>> {
        wx_core_error(|| {
            let sql = "SELECT * FROM FavItem WHERE localId = ?";
            self.db.execute_query_one(sql, &[&favorite_id])
        })
    }

    /// Search favorites
    pub fn search_favorites(
        &self,
        keyword: &str,
        limit: usize,
        offset: usize,
    ) -> WxCoreResult<Vec<serde_json::Value>> {
        wx_core_error(|| {
            let sql = format!(
                "SELECT * FROM FavItem 
                WHERE content LIKE ? 
                ORDER BY localId DESC 
                LIMIT {} OFFSET {}",
                limit, offset
            );

            let keyword = format!("%{}%", keyword);
            self.db.execute_query(&sql, &[&keyword])
        })
    }

    /// Get favorite count
    pub fn get_favorite_count(&self) -> WxCoreResult<i64> {
        wx_core_error(|| {
            let sql = "SELECT COUNT(*) as count FROM FavItem";
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

    /// Get favorite by type
    pub fn get_favorite_by_type(
        &self,
        favorite_type: i64,
        limit: usize,
        offset: usize,
    ) -> WxCoreResult<Vec<serde_json::Value>> {
        wx_core_error(|| {
            let sql = format!(
                "SELECT * FROM FavItem 
                WHERE type = ? 
                ORDER BY localId DESC 
                LIMIT {} OFFSET {}",
                limit, offset
            );

            self.db.execute_query(&sql, &[&favorite_type])
        })
    }

    /// Get favorite count by type
    pub fn get_favorite_count_by_type(&self, favorite_type: i64) -> WxCoreResult<i64> {
        wx_core_error(|| {
            let sql = "SELECT COUNT(*) as count FROM FavItem WHERE type = ?";
            let result = self.db.execute_query_one(sql, &[&favorite_type])?;

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
