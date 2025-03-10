use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Database type constants
pub const DB_TYPE_MSG: &str = "MSG";
pub const DB_TYPE_MICRO_MSG: &str = "MicroMsg";
pub const DB_TYPE_MEDIA_MSG: &str = "MediaMSG";
pub const DB_TYPE_OPENIM_CONTACT: &str = "OpenIMContact";
pub const DB_TYPE_OPENIM_MEDIA: &str = "OpenIMMedia";
pub const DB_TYPE_FAVORITE: &str = "Favorite";
pub const DB_TYPE_PUBLIC_MSG: &str = "PublicMsg";
pub const DB_TYPE_SNS: &str = "Sns";

/// Message type constants
pub const MSG_TYPE_TEXT: i64 = 1;
pub const MSG_TYPE_IMAGE: i64 = 3;
pub const MSG_TYPE_VOICE: i64 = 34;
pub const MSG_TYPE_VIDEO: i64 = 43;
pub const MSG_TYPE_MICROVIDEO: i64 = 62;
pub const MSG_TYPE_EMOTICON: i64 = 47;
pub const MSG_TYPE_APP: i64 = 49;
pub const MSG_TYPE_FILE: i64 = 49 + 6;
pub const MSG_TYPE_SYSTEM: i64 = 10000;
pub const MSG_TYPE_RECALLED: i64 = 10002;

/// Get message type name
pub fn get_msg_type_name(msg_type: i64) -> &'static str {
    match msg_type {
        MSG_TYPE_TEXT => "Text",
        MSG_TYPE_IMAGE => "Image",
        MSG_TYPE_VOICE => "Voice",
        MSG_TYPE_VIDEO => "Video",
        MSG_TYPE_MICROVIDEO => "MicroVideo",
        MSG_TYPE_EMOTICON => "Emoticon",
        MSG_TYPE_APP => "App",
        MSG_TYPE_FILE => "File",
        MSG_TYPE_SYSTEM => "System",
        MSG_TYPE_RECALLED => "Recalled",
        _ => "Unknown",
    }
}

/// Get database type name
pub fn get_db_type_name(db_type: &str) -> &'static str {
    match db_type {
        DB_TYPE_MSG => "Message Database",
        DB_TYPE_MICRO_MSG => "Contact Database",
        DB_TYPE_MEDIA_MSG => "Media Database",
        DB_TYPE_OPENIM_CONTACT => "OpenIM Contact Database",
        DB_TYPE_OPENIM_MEDIA => "OpenIM Media Database",
        DB_TYPE_FAVORITE => "Favorite Database",
        DB_TYPE_PUBLIC_MSG => "Public Message Database",
        DB_TYPE_SNS => "Moments Database",
        _ => "Unknown Database",
    }
}

/// Check if a file is a SQLite database
pub fn is_sqlite_db(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();

    if !path.exists() || !path.is_file() {
        return false;
    }

    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return false,
    };

    let mut header = [0u8; 16];
    if let Err(_) = file.read_exact(&mut header) {
        return false;
    }

    // Check for SQLite header
    let header_str = String::from_utf8_lossy(&header);
    header_str.starts_with("SQLite format 3")
}

/// Get database type from file path
pub fn get_db_type_from_path(path: impl AsRef<Path>) -> Option<String> {
    let path = path.as_ref();

    if !is_sqlite_db(path) {
        return None;
    }

    let file_name = path.file_name()?.to_string_lossy();

    // Extract database type from file name
    // For example, "MSG.db" -> "MSG", "MicroMsg.db" -> "MicroMsg"
    let db_type = if file_name.ends_with(".db") {
        let name = file_name.trim_end_matches(".db");
        // Remove any numbers at the end
        let re = regex::Regex::new(r"\d+$").unwrap();
        re.replace(name, "").to_string()
    } else {
        file_name.to_string()
    };

    Some(db_type)
}

/// Get database handler type from database type
pub fn get_db_handler_type(db_type: &str) -> &'static str {
    match db_type {
        DB_TYPE_MSG => "MsgHandler",
        DB_TYPE_MICRO_MSG => "MicroHandler",
        DB_TYPE_MEDIA_MSG => "MediaHandler",
        DB_TYPE_OPENIM_CONTACT => "OpenIMContactHandler",
        DB_TYPE_OPENIM_MEDIA => "OpenIMMediaHandler",
        DB_TYPE_FAVORITE => "FavoriteHandler",
        DB_TYPE_PUBLIC_MSG => "PublicMsgHandler",
        DB_TYPE_SNS => "SnsHandler",
        _ => "DBHandler",
    }
}
