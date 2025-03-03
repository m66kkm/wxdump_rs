use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Write};

use crate::wx_core::utils::{WxCoreResult, wx_core_error};
use crate::db::MsgHandler;

/// Export chat messages to HTML
pub fn export_html(
    db_path: impl AsRef<Path>,
    chat_id: &str,
    output_path: impl AsRef<Path>,
    my_wxid: Option<&str>,
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
        
        // Generate HTML
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html>\n");
        html.push_str("<head>\n");
        html.push_str("  <meta charset=\"UTF-8\">\n");
        html.push_str("  <title>Chat Export</title>\n");
        html.push_str("  <style>\n");
        html.push_str("    body { font-family: Arial, sans-serif; margin: 0; padding: 20px; }\n");
        html.push_str("    .message { margin-bottom: 10px; padding: 10px; border-radius: 5px; max-width: 70%; }\n");
        html.push_str("    .sent { background-color: #DCF8C6; margin-left: auto; }\n");
        html.push_str("    .received { background-color: #F1F0F0; margin-right: auto; }\n");
        html.push_str("    .message-container { display: flex; flex-direction: column; }\n");
        html.push_str("    .timestamp { font-size: 0.8em; color: #999; margin-top: 5px; }\n");
        html.push_str("  </style>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");
        html.push_str("  <div class=\"message-container\">\n");
        
        for message in messages {
            if let serde_json::Value::Object(map) = message {
                let talker = map.get("talker").and_then(|v| v.as_str()).unwrap_or("");
                let content = map.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let create_time = map.get("createTime").and_then(|v| v.as_i64()).unwrap_or(0);
                
                let is_sent = if let Some(my_id) = my_wxid {
                    talker != my_id
                } else {
                    false
                };
                
                let message_class = if is_sent { "sent" } else { "received" };
                let timestamp = chrono::DateTime::from_timestamp(create_time, 0)
                    .map(|dt| dt.naive_local().format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| create_time.to_string());
                
                html.push_str(&format!("    <div class=\"message {}\">\n", message_class));
                html.push_str(&format!("      <div>{}</div>\n", content));
                html.push_str(&format!("      <div class=\"timestamp\">{}</div>\n", timestamp));
                html.push_str("    </div>\n");
            }
        }
        
        html.push_str("  </div>\n");
        html.push_str("</body>\n");
        html.push_str("</html>\n");
        
        // Write HTML to file
        let mut file = File::create(output_path)?;
        file.write_all(html.as_bytes())?;
        
        Ok(output_path.to_path_buf())
    })
}
