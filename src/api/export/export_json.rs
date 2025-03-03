use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Write};

use crate::wx_core::utils::{WxCoreResult, wx_core_error};
use crate::db::MsgHandler;

/// Export chat messages to JSON
pub fn export_json(
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
        
        // Create JSON file
        let mut file = File::create(output_path)?;
        
        // Write JSON
        let json = serde_json::to_string_pretty(&messages)?;
        file.write_all(json.as_bytes())?;
        
        Ok(output_path.to_path_buf())
    })
}
