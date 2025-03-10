use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::db::MsgHandler;
use crate::wx_core::utils::{wx_core_error, WxCoreResult};

/// Export chat messages to CSV
pub fn export_csv(
    db_path: impl AsRef<Path>,
    chat_id: &str,
    output_path: impl AsRef<Path>,
) -> WxCoreResult<PathBuf> {
    wx_core_error(|| {
        let db_path = db_path.as_ref();
        let output_path = output_path.as_ref();

        // Create output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // Open database
        let msg_handler = MsgHandler::new(db_path)?;

        // Get chat messages
        let messages = msg_handler.get_chat_messages(chat_id, 1000, 0)?;

        // Create CSV file
        let mut file = File::create(output_path)?;

        // Write CSV header
        writeln!(file, "msgId,talker,content,createTime,type")?;

        // Write CSV rows
        for message in messages {
            if let serde_json::Value::Object(map) = message {
                let msg_id = map.get("msgId").and_then(|v| v.as_i64()).unwrap_or(0);
                let talker = map.get("talker").and_then(|v| v.as_str()).unwrap_or("");
                let content = map.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let create_time = map.get("createTime").and_then(|v| v.as_i64()).unwrap_or(0);
                let msg_type = map.get("type").and_then(|v| v.as_i64()).unwrap_or(0);

                // Escape CSV special characters
                let content = content.replace("\"", "\"\"");

                writeln!(
                    file,
                    "{},\"{}\",\"{}\",{},{}",
                    msg_id, talker, content, create_time, msg_type
                )?;
            }
        }

        Ok(output_path.to_path_buf())
    })
}
