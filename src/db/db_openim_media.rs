use std::path::{Path, PathBuf};

use crate::db::db_base::DBHandler;
use crate::wx_core::utils::{wx_core_error, WxCoreResult};

/// OpenIMMedia database handler
pub struct OpenIMMediaHandler {
    pub db: DBHandler,
}

impl OpenIMMediaHandler {
    /// Create a new OpenIMMedia database handler
    pub fn new(db_path: impl AsRef<Path>) -> WxCoreResult<Self> {
        wx_core_error(|| {
            let db = DBHandler::new(db_path)?;
            Ok(Self { db })
        })
    }

    /// Get media by message ID
    pub fn get_media_by_msg_id(&self, msg_id: i64) -> WxCoreResult<Option<serde_json::Value>> {
        wx_core_error(|| {
            let sql = "SELECT * FROM OpenIMMedia WHERE MsgId = ?";
            self.db.execute_query_one(sql, &[&msg_id])
        })
    }

    /// Get media by chat ID
    pub fn get_media_by_chat_id(
        &self,
        chat_id: &str,
        limit: usize,
        offset: usize,
    ) -> WxCoreResult<Vec<serde_json::Value>> {
        wx_core_error(|| {
            let sql = format!(
                "SELECT * FROM OpenIMMedia WHERE TalkerId = ? ORDER BY CreateTime DESC LIMIT {} OFFSET {}",
                limit, offset
            );

            self.db.execute_query(&sql, &[&chat_id])
        })
    }

    /// Get media by type
    pub fn get_media_by_type(
        &self,
        media_type: i64,
        limit: usize,
        offset: usize,
    ) -> WxCoreResult<Vec<serde_json::Value>> {
        wx_core_error(|| {
            let sql = format!(
                "SELECT * FROM OpenIMMedia WHERE Type = ? ORDER BY CreateTime DESC LIMIT {} OFFSET {}",
                limit, offset
            );

            self.db.execute_query(&sql, &[&media_type])
        })
    }

    /// Get media count
    pub fn get_media_count(&self) -> WxCoreResult<i64> {
        wx_core_error(|| {
            let sql = "SELECT COUNT(*) as count FROM OpenIMMedia";
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

    /// Get media count by type
    pub fn get_media_count_by_type(&self, media_type: i64) -> WxCoreResult<i64> {
        wx_core_error(|| {
            let sql = "SELECT COUNT(*) as count FROM OpenIMMedia WHERE Type = ?";
            let result = self.db.execute_query_one(sql, &[&media_type])?;

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

    /// Get media file path
    pub fn get_media_file_path(
        &self,
        msg_id: i64,
        wx_path: Option<&Path>,
    ) -> WxCoreResult<Option<PathBuf>> {
        wx_core_error(|| {
            let sql = "SELECT Path FROM OpenIMMedia WHERE MsgId = ?";
            let result = self.db.execute_query_one(sql, &[&msg_id])?;

            if let Some(serde_json::Value::Object(map)) = result {
                if let Some(serde_json::Value::String(path)) = map.get("Path") {
                    if let Some(wx_path) = wx_path {
                        // Combine the WeChat path with the media path
                        return Ok(Some(wx_path.join(path)));
                    } else {
                        // Return the media path as-is
                        return Ok(Some(PathBuf::from(path)));
                    }
                }
            }

            Ok(None)
        })
    }

    /// Close the database connection
    pub fn close(self) -> WxCoreResult<()> {
        self.db.close()
    }
}
