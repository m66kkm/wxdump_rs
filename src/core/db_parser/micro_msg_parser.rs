use anyhow::Result;
use std::collections::HashMap;
use std::fmt; // Added for Display

use chrono::{DateTime, NaiveDateTime, Utc};
use rusqlite::{Connection, Result as RusqliteResult};

// Custom error type to wrap anyhow::Error for std::error::Error compatibility
#[derive(Debug)]
struct AnyhowToStdError(String);

impl fmt::Display for AnyhowToStdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AnyhowToStdError {}
#[derive(Debug, Default, Clone)]
pub struct ExtraBufInfo {
    pub gender: Option<i64>,
    pub signature: Option<String>,
    pub country: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
    pub company_name: Option<String>,
    pub mobile_phone: Option<String>,
    pub enterprise_wechat_attr: Option<String>,
    pub moments_background_img: Option<String>,
    pub remark_img_url1: Option<String>,
    pub remark_img_url2: Option<String>,
    // TODO: Add other fields from buf_dict if needed
}
#[derive(Debug, Clone, Default)]
pub struct SessionInfo {
    pub wxid: String,
    pub order_num: Option<i64>,
    pub unread_count: Option<i64>,
    pub session_nickname: Option<String>,
    pub session_status: Option<i64>,
    pub is_send: Option<i64>,
    pub content: Option<String>,
    pub msg_local_id: Option<i64>,
    pub msg_status: Option<i64>,
    pub timestamp: Option<i64>,
    pub time_str: Option<String>,
    pub msg_type: Option<i64>,
    pub msg_sub_type: Option<i64>,
    pub contact_nickname: Option<String>,
    pub contact_remark: Option<String>,
    pub contact_account: Option<String>,
    pub contact_description: Option<String>,
    pub contact_head_img_url: Option<String>,
    pub contact_extra_buf_info: Option<ExtraBufInfo>,
    pub contact_label_list: Vec<String>,
    pub contact_del_flag: Option<i64>,
    pub contact_type: Option<i64>,
    pub contact_verify_flag: Option<i64>,
    pub contact_chat_room_type: Option<i64>,
    pub contact_chat_room_notify: Option<i64>,
}

pub fn format_timestamp_to_string(timestamp_secs: i64, format_str: &str) -> String {
    if let Some(naive_dt) = NaiveDateTime::from_timestamp_opt(timestamp_secs, 0) {
        let datetime_utc: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive_dt, Utc);
        datetime_utc.format(format_str).to_string()
    } else {
        // Return a default string or an empty string if the timestamp is invalid
        // For simplicity, returning an empty string.
        // Consider returning Result<String, _> for better error handling in a real app.
        "".to_string() 
        // Or: "Invalid Timestamp".to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Contact {
    pub wxid: String,
    pub account: Option<String>,
    pub nickname: Option<String>,
    pub remark: Option<String>,
    pub head_img_url: Option<String>,
    pub label_list: Vec<String>,
    pub description: Option<String>,
    pub extra_buf_info: Option<ExtraBufInfo>,
    pub user_type: Option<i64>,
    pub verify_flag: Option<i64>,
    pub chat_room_type: Option<i64>,
    pub del_flag: Option<i64>,
    pub reserved1: Option<i64>, // Typically gender
    pub reserved2: Option<i64>,
    pub reserved5: Option<i64>,
    pub chat_room_notify: Option<i64>,
    pub is_chatroom_contact: bool,
}
#[derive(Debug, Clone, Default)]
pub struct ChatRoomMember {
    pub wxid: String,
    pub nickname: Option<String>,
    pub remark: Option<String>,
    pub account: Option<String>,
    pub head_img_url: Option<String>,
    pub room_nickname: Option<String>, // From RoomData parsing
}

#[derive(Debug, Clone, Default)]
pub struct ChatRoomInfo {
    pub wxid: String,                         // From ChatRoomName
    pub member_wxids: Vec<String>,            // From UserNameList, split
    pub self_display_name: Option<String>,    // From SelfDisplayName
    pub owner_wxid: Option<String>,           // From Reserved2 (ChatRoom table)
    pub announcement: Option<String>,         // From Announcement (ChatRoomInfo table)
    pub announcement_editor: Option<String>,  // From AnnouncementEditor
    pub announcement_publish_time: Option<i64>, // From AnnouncementPublishTime
    pub members: Vec<ChatRoomMember>,         // Populated via get_contacts and parse_chat_room_data
    pub is_show_name: Option<i64>,            // From IsShowName
    pub chat_room_flag: Option<i64>,          // From ChatRoomFlag
    // RoomData parsing result can be temporarily stored or used to populate members' room_nickname
}

/// Parses the RoomData field from the ChatRoom table.
///
/// TODO: Implement proper protobuf parsing for RoomData.
/// The Python code uses blackboxprotobuf.decode_message.
/// For now, this is a placeholder.
pub fn parse_chat_room_data(_room_data_bytes: Option<&[u8]>) -> Result<HashMap<String, String>, anyhow::Error> {
    // Placeholder implementation
    // In Python, this parses protobuf data to get wxid -> roomNickname mappings.
    // e.g., i['1'] (wxid) and i['2'] (roomNickname)
    Ok(HashMap::new())
    // Or, to indicate it's not implemented:
    // Err(anyhow::anyhow!("RoomData parsing not yet implemented"))
}

/// Retrieves information about chat rooms.
/// Corresponds to Python's `get_room_list`.
pub fn get_chat_rooms(
    conn: &Connection,
    filter_room_wxids: Option<&[String]>,
) -> Result<HashMap<String, ChatRoomInfo>, anyhow::Error> {
    let mut sql = String::from(
        "SELECT A.ChatRoomName, A.UserNameList, A.SelfDisplayName, A.Reserved2 AS owner_wxid, \
         A.RoomData, A.IsShowName, A.ChatRoomFlag, \
         B.Announcement, B.AnnouncementEditor, B.AnnouncementPublishTime \
         FROM ChatRoom A LEFT JOIN ChatRoomInfo B ON A.ChatRoomName = B.ChatRoomName",
    );

    let mut conditions: Vec<String> = Vec::new();
    let mut params_list: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(wxids) = filter_room_wxids {
        if !wxids.is_empty() {
            let placeholders = wxids.iter().map(|_| "?").collect::<Vec<&str>>().join(",");
            conditions.push(format!("A.ChatRoomName IN ({})", placeholders));
            for wxid in wxids {
                params_list.push(Box::new(wxid.clone()));
            }
        } else {
            // If wxids is an empty list, no results should match
            conditions.push("1=0".to_string());
        }
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }
    sql.push_str(";");

    let params_for_query: Vec<&dyn rusqlite::ToSql> = params_list.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;

    let mut chat_room_map = HashMap::new();

    let rows = stmt.query_map(&*params_for_query, |row| {
        let chat_room_name: String = row.get("ChatRoomName")?;
        let user_name_list_opt: Option<String> = row.get("UserNameList")?;
        let room_data_bytes: Option<Vec<u8>> = row.get("RoomData")?;

        let member_wxids: Vec<String> = user_name_list_opt
            .map(|s| {
                s.split(|c| c == ',' || c == '\x07') // Split by comma or ASCII BEL
                    .filter(|id| !id.is_empty())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        // Parse RoomData (currently a placeholder)
        let room_nicknames_map = parse_chat_room_data(room_data_bytes.as_deref())
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                0, rusqlite::types::Type::Blob, Box::new(AnyhowToStdError(e.to_string()))
            ))?;

        // Get contact details for members
        let mut chat_room_members: Vec<ChatRoomMember> = Vec::new();
        if !member_wxids.is_empty() {
            // Convert Vec<String> to &[String] for get_contacts
            let member_wxid_slices: Vec<String> = member_wxids.iter().map(|s| s.to_string()).collect();

            match get_contacts(conn, None, Some(&member_wxid_slices), None) {
                Ok(contacts) => {
                    for contact in contacts {
                        chat_room_members.push(ChatRoomMember {
                            wxid: contact.wxid.clone(),
                            nickname: contact.nickname.clone(),
                            remark: contact.remark.clone(),
                            account: contact.account.clone(),
                            head_img_url: contact.head_img_url.clone(),
                            room_nickname: room_nicknames_map.get(&contact.wxid).cloned(),
                        });
                    }
                }
                Err(e) => {
                     // Log or handle error from get_contacts
                    eprintln!("Error fetching members for room {}: {}", chat_room_name, e);
                    // Convert anyhow::Error to rusqlite::Error to propagate
                    return Err(rusqlite::Error::FromSqlConversionFailure(
                        0, rusqlite::types::Type::Null, Box::new(AnyhowToStdError(e.to_string()))
                    ));
                }
            }
        }

        Ok(ChatRoomInfo {
            wxid: chat_room_name,
            member_wxids,
            self_display_name: row.get("SelfDisplayName")?,
            owner_wxid: row.get("owner_wxid")?,
            announcement: row.get("Announcement")?,
            announcement_editor: row.get("AnnouncementEditor")?,
            announcement_publish_time: row.get("AnnouncementPublishTime")?,
            members: chat_room_members,
            is_show_name: row.get("IsShowName")?,
            chat_room_flag: row.get("ChatRoomFlag")?,
        })
    })?;

    for row_result in rows {
        match row_result {
            Ok(chat_room_info) => {
                chat_room_map.insert(chat_room_info.wxid.clone(), chat_room_info);
            }
            Err(e) => {
                // Handle or propagate the error from row mapping
                // For simplicity, we'll print and continue, but a robust app might return Err here.
                eprintln!("Error processing chat room row: {}", e);
                // Or, to propagate: return Err(anyhow::anyhow!("Failed to process row: {}", e));
            }
        }
    }

    Ok(chat_room_map)
}

#[allow(dead_code)] // Placeholder for now
enum ExpectedType {
    Int,
    Utf16String,
    Utf8String,
    HexBytes,
}

// Placeholder for the buf_dict mapping
// The actual mapping will be more complex and involve parsing logic.
#[allow(dead_code)] // Placeholder for now
fn get_buf_map() -> HashMap<&'static str, (&'static str, ExpectedType)> {
    let mut map = HashMap::new();
    // Example entry, will be populated based on Python's buf_dict
    // map.insert("74752C06", ("gender", ExpectedType::Int));
    map
}

pub fn parse_extra_buf(extra_buf_bytes: Option<&[u8]>) -> Result<Option<ExtraBufInfo>> {
    let bytes = match extra_buf_bytes {
        Some(b) if !b.is_empty() => b,
        _ => return Ok(None),
    };

    let mut info = ExtraBufInfo::default();
    // The buf_dict from Python's get_ExtraBuf function
    // buf_dict = {
    //     '74752C06': ('gender', 2, 1, 1),  # 性别
    //     '46CF10C4': ('signature', 2, 2, 2),  # 个性签名
    //     'A4D9024A': ('country', 2, 2, 2),  # 国家
    //     'E2EAA8D1': ('province', 2, 2, 2),  # 省份
    //     '1D025BBF': ('city', 2, 2, 2),  # 城市
    //     'F917BCC0': ('company_name', 2, 2, 2),  # 公司名称
    //     '759378AD': ('mobile_phone', 2, 2, 2),  # 手机号
    //     '4EB96D85': ('enterprise_wechat_attr', 2, 2, 2),  # 企微属性
    //     '81AE19B4': ('moments_background_img', 2, 2, 2),  # 朋友圈背景图
    //     '0E719F13': ('remark_img_url1', 2, 2, 2),  # 备注图片1
    //     '945f3190': ('remark_img_url2', 2, 2, 2),  # 备注图片2
    //     # ... other fields
    // }
    // For simplicity, we'll manually define the parsing logic for each known field
    // A more robust solution would involve a loop and a map similar to Python's buf_dict

    // Helper function to find and parse a value
    fn find_and_parse_string(bytes: &[u8], key_hex: &str, field_name: &str) -> Result<Option<String>> {
        let key = hex::decode(key_hex).map_err(|e| anyhow::anyhow!("Failed to decode hex key {}: {}", key_hex, e))?;
        if let Some(start_index) = bytes.windows(key.len()).position(|window| window == key) {
            let data_start = start_index + key.len();
            // Assuming type_id is 1 byte, length is 2 bytes (u16 little endian)
            if data_start + 3 <= bytes.len() {
                // let type_id = bytes[data_start]; // type_id = 2 for string
                let len = u16::from_le_bytes([bytes[data_start + 1], bytes[data_start + 2]]) as usize;
                let value_start = data_start + 3;
                if value_start + len <= bytes.len() {
                    let value_bytes = &bytes[value_start..value_start + len];
                    // Assuming UTF-16LE based on common WeChat patterns, adjust if needed
                    // Python code uses `value.decode('utf-16', 'ignore')`
                    // For Rust, we might need to handle potential errors more explicitly or use a lossy conversion.
                    // For now, let's try utf-16. If it's utf-8, the python code would be different.
                    // The python code uses `value.decode('utf-16', 'ignore')` for type_id == 2
                    // and `value.decode('utf-8', 'ignore')` for type_id == 3
                    // The provided buf_dict in python has type_id = 2 for strings.
                    let utf16_chars: Vec<u16> = value_bytes
                        .chunks_exact(2)
                        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                        .collect();
                    match String::from_utf16(&utf16_chars) {
                        Ok(s) => return Ok(Some(s)),
                        Err(e) => {
                            // Fallback or log error
                            eprintln!("Failed to decode UTF-16 for {}: {}", field_name, e);
                            // Try UTF-8 as a fallback, though less likely for type_id 2
                            match String::from_utf8(value_bytes.to_vec()) {
                                Ok(s_utf8) => {
                                    eprintln!("Successfully decoded as UTF-8 for {} (fallback)", field_name);
                                    return Ok(Some(s_utf8));
                                }
                                Err(e_utf8) => {
                                    eprintln!("Failed to decode UTF-8 for {} (fallback): {}", field_name, e_utf8);
                                    return Ok(None); // Or handle error differently
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    fn find_and_parse_i64(bytes: &[u8], key_hex: &str, _field_name: &str) -> Result<Option<i64>> {
        let key = hex::decode(key_hex).map_err(|e| anyhow::anyhow!("Failed to decode hex key {}: {}", key_hex, e))?;
        if let Some(start_index) = bytes.windows(key.len()).position(|window| window == key) {
            let data_start = start_index + key.len();
            // Assuming type_id is 1 byte, length is 1 byte (for i64, it's usually fixed or indicated by length)
            // Python code: value = int.from_bytes(buf[pos + 1 + 2: pos + 1 + 2 + length], "little")
            // This implies length is also read. For type_id = 1 (int), length is 1 byte.
            if data_start + 2 <= bytes.len() {
                // let type_id = bytes[data_start]; // type_id = 1 for int
                let length = bytes[data_start + 1] as usize;
                let value_start = data_start + 2;
                if value_start + length <= bytes.len() && length <= 8 { // Max 8 bytes for i64
                    let value_bytes = &bytes[value_start..value_start + length];
                    let mut val_arr = [0u8; 8];
                    val_arr[..length].copy_from_slice(value_bytes);
                    return Ok(Some(i64::from_le_bytes(val_arr)));
                }
            }
        }
        Ok(None)
    }

    info.gender = find_and_parse_i64(bytes, "74752C06", "gender")?;
    info.signature = find_and_parse_string(bytes, "46CF10C4", "signature")?;
    info.country = find_and_parse_string(bytes, "A4D9024A", "country")?;
    info.province = find_and_parse_string(bytes, "E2EAA8D1", "province")?;
    info.city = find_and_parse_string(bytes, "1D025BBF", "city")?;
    info.company_name = find_and_parse_string(bytes, "F917BCC0", "company_name")?;
    info.mobile_phone = find_and_parse_string(bytes, "759378AD", "mobile_phone")?;
    info.enterprise_wechat_attr = find_and_parse_string(bytes, "4EB96D85", "enterprise_wechat_attr")?;
    info.moments_background_img = find_and_parse_string(bytes, "81AE19B4", "moments_background_img")?;
    info.remark_img_url1 = find_and_parse_string(bytes, "0E719F13", "remark_img_url1")?;
    info.remark_img_url2 = find_and_parse_string(bytes, "945f3190", "remark_img_url2")?;


    Ok(Some(info))
}
pub fn get_contact_labels(conn: &Connection) -> RusqliteResult<HashMap<i64, String>> {
    let mut stmt = conn.prepare("SELECT LabelId, LabelName FROM ContactLabel ORDER BY LabelName ASC;")?;
    let label_iter = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?;

    let mut labels = HashMap::new();
    for label_result in label_iter {
        let (id, name): (i64, String) = label_result?;
        labels.insert(id, name);
    }
    Ok(labels)
}
pub fn get_contacts(
    conn: &Connection,
    filter_word: Option<&str>,
    filter_wxids: Option<&[String]>,
    filter_label_ids: Option<&[i64]>,
) -> Result<Vec<Contact>> {
    let label_map = get_contact_labels(conn).map_err(|e| anyhow::anyhow!("Failed to get contact labels: {}", e))?;

    let mut sql = String::from(
        "SELECT A.UserName, A.Alias, A.NickName, A.Remark, A.LabelIDList, \
         A.Reserved6 AS description, A.ExtraBuf, A.Type, A.VerifyFlag, \
         A.ChatRoomType, A.DelFlag, A.Reserved1, A.Reserved2, A.Reserved5, \
         A.ChatRoomNotify, B.bigHeadImgUrl \
         FROM Contact A LEFT JOIN ContactHeadImgUrl B ON A.UserName = B.usrName",
    );

    let mut conditions: Vec<String> = Vec::new();
    let mut params_list: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(word) = filter_word {
        let like_pattern = format!("%{}%", word);
        let or_conditions: Vec<String> = [
            "LOWER(A.UserName) LIKE LOWER(?)",
            "LOWER(A.NickName) LIKE LOWER(?)",
            "LOWER(A.Remark) LIKE LOWER(?)",
            "LOWER(A.Alias) LIKE LOWER(?)",
            "LOWER(A.QuanPin) LIKE LOWER(?)",
            "LOWER(A.PYInitial) LIKE LOWER(?)",
            "LOWER(A.RemarkQuanPin) LIKE LOWER(?)",
            "LOWER(A.RemarkPYInitial) LIKE LOWER(?)",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        conditions.push(format!("({})", or_conditions.join(" OR ")));
        for _ in 0..or_conditions.len() {
            params_list.push(Box::new(like_pattern.clone()));
        }
    }

    if let Some(wxids) = filter_wxids {
        if !wxids.is_empty() {
            let placeholders = wxids.iter().map(|_| "?").collect::<Vec<&str>>().join(",");
            conditions.push(format!("A.UserName IN ({})", placeholders));
            for wxid in wxids {
                params_list.push(Box::new(wxid.clone()));
            }
        } else {
            // If wxids is an empty list, no results should match
            conditions.push("1=0".to_string());
        }
    }

    if let Some(label_ids) = filter_label_ids {
        if !label_ids.is_empty() {
            let label_conditions: Vec<String> = label_ids
                .iter()
                .map(|id| {
                    params_list.push(Box::new(format!("%{}%", id)));
                    "A.LabelIDList LIKE ?".to_string()
                })
                .collect();
            conditions.push(format!("({})", label_conditions.join(" OR ")));
        } else {
             // If label_ids is an empty list, no results should match this specific filter part
            conditions.push("1=0".to_string());
        }
    }
    
    // Add a general condition to filter out some system contacts, if not already filtered by wxid
    // This is a common practice, adjust as needed.
    if filter_wxids.is_none() {
        conditions.push("A.UserName NOT LIKE '%@app'".to_string());
        conditions.push("A.UserName NOT LIKE '%@chatroom'".to_string()); // Assuming get_contacts is for individual users primarily
        conditions.push("A.Type != 4".to_string()); // Type 4 are often special/system contacts
        conditions.push("A.Type != 0".to_string()); // Type 0 can be current user or system accounts
    }


    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" ORDER BY A.RemarkPYInitial, A.PYInitial, A.NickName;");

    // Convert Vec<Box<dyn ToSql>> to Vec<&dyn ToSql> for rusqlite::params_from_iter
    let params_for_query: Vec<&dyn rusqlite::ToSql> = params_list.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let contact_iter = stmt.query_map(&*params_for_query, |row| {
        let wxid: String = row.get("UserName")?;
        let label_id_list_str: Option<String> = row.get("LabelIDList")?;
        let mut labels = Vec::new();
        if let Some(ids_str) = label_id_list_str {
            for id_str in ids_str.split(',') {
                if let Ok(id) = id_str.trim().parse::<i64>() {
                    if let Some(name) = label_map.get(&id) {
                        labels.push(name.clone());
                    } else {
                        // labels.push(format!("id_{}", id)); // Optionally add raw id if name not found
                    }
                }
            }
        }

        let extra_buf_bytes: Option<Vec<u8>> = row.get("ExtraBuf")?;
        let extra_buf_info = match parse_extra_buf(extra_buf_bytes.as_deref()) {
            Ok(info) => info,
            Err(e) => {
                // Convert anyhow::Error to rusqlite::Error::FromSqlConversionFailure
                let column_index = 0; // Placeholder, as this isn't a direct SQL column conversion
                let source_type = rusqlite::types::Type::Blob; // ExtraBuf is likely a BLOB
                
                // Wrap the anyhow::Error's string representation in our custom error type
                let std_error = Box::new(AnyhowToStdError(e.to_string()));

                return Err(rusqlite::Error::FromSqlConversionFailure(
                    column_index,
                    source_type,
                    std_error, // This now correctly implements std::error::Error
                ));
            }
        };

        let is_chatroom_contact = wxid.contains("@chatroom");

        Ok(Contact {
            wxid,
            account: row.get("Alias")?, // Python's 'Alias' seems to map to 'account'
            nickname: row.get("NickName")?,
            remark: row.get("Remark")?,
            head_img_url: row.get("bigHeadImgUrl")?,
            label_list: labels,
            description: row.get("description")?,
            extra_buf_info,
            user_type: row.get("Type")?,
            verify_flag: row.get("VerifyFlag")?,
            chat_room_type: row.get("ChatRoomType")?,
            del_flag: row.get("DelFlag")?,
            reserved1: row.get("Reserved1")?,
            reserved2: row.get("Reserved2")?,
            reserved5: row.get("Reserved5")?,
            chat_room_notify: row.get("ChatRoomNotify")?,
            is_chatroom_contact,
        })
    })?;

    let mut contacts = Vec::new();
    for contact_result in contact_iter {
        contacts.push(contact_result?);
    }

    Ok(contacts)
}

pub fn get_sessions(conn: &Connection) -> Result<Vec<SessionInfo>, anyhow::Error> {
    let label_map = get_contact_labels(conn)
        .map_err(|e| anyhow::anyhow!("Failed to get contact labels: {}", e))?;

    let sql = r#"
SELECT
    S.strUsrName, S.nOrder, S.nUnReadCount, S.strNickName AS session_str_nick_name,
    S.nStatus, S.nIsSend, S.strContent, S.nMsgLocalID, S.nMsgStatus, S.nTime,
    S.nMsgType, S.Reserved2 AS session_reserved2_msg_sub_type,
    C.UserName AS contact_user_name, C.Alias AS contact_alias, C.DelFlag AS contact_del_flag,
    C.Type AS contact_type, C.VerifyFlag AS contact_verify_flag,
    C.Reserved1 AS contact_reserved1_gender, C.Reserved2 AS contact_reserved2,
    C.Remark AS contact_remark, C.NickName AS contact_nick_name,
    C.LabelIDList AS contact_label_id_list, C.ChatRoomType AS contact_chat_room_type,
    C.ChatRoomNotify AS contact_chat_room_notify, C.Reserved5 AS contact_reserved5,
    C.Reserved6 AS contact_reserved6_describe, C.ExtraBuf AS contact_extra_buf,
    H.bigHeadImgUrl AS contact_big_head_img_url
FROM
    Session S
INNER JOIN
    (SELECT strUsrName, MAX(nTime) AS MaxnTime FROM Session GROUP BY strUsrName) AS SubQuery
ON
    S.strUsrName = SubQuery.strUsrName AND S.nTime = SubQuery.MaxnTime
INNER JOIN
    Contact C ON S.strUsrName = C.UserName
LEFT JOIN
    ContactHeadImgUrl H ON C.UserName = H.usrName
WHERE
    S.strUsrName != '@publicUser'
ORDER BY
    S.nTime DESC;
    "#;

    let mut stmt = conn.prepare(sql)?;

    let mapped_rows = stmt.query_map([], |row| {
        let wxid: String = row.get("strUsrName")?;
        
        let timestamp_opt: Option<i64> = row.get("nTime")?;
        let time_str: Option<String> = timestamp_opt.map(|ts| format_timestamp_to_string(ts, "%Y-%m-%d %H:%M:%S"));

        let label_id_list_str: Option<String> = row.get("contact_label_id_list")?;
        let mut contact_label_list = Vec::new();
        if let Some(ids_str) = label_id_list_str {
            if !ids_str.is_empty() {
                for id_str in ids_str.split(',') {
                    if let Ok(id) = id_str.trim().parse::<i64>() {
                        if let Some(name) = label_map.get(&id) {
                            contact_label_list.push(name.clone());
                        }
                    }
                }
            }
        }

        let contact_extra_buf_bytes: Option<Vec<u8>> = row.get("contact_extra_buf")?;
        let contact_extra_buf_info = match parse_extra_buf(contact_extra_buf_bytes.as_deref()) {
            Ok(info) => info,
            Err(e) => {
                eprintln!("Error parsing ExtraBuf for session with wxid {}: {}", wxid, e);
                // Convert anyhow::Error to rusqlite::Error to satisfy query_map's error type
                return Err(rusqlite::Error::FromSqlConversionFailure(
                    0, // Replaced problematic column_index call with a fixed value
                    rusqlite::types::Type::Blob,
                    Box::new(AnyhowToStdError(format!("Failed to parse ExtraBuf for {}: {}", wxid, e)))
                ));
            }
        };

        Ok(SessionInfo {
            wxid,
            order_num: row.get("nOrder")?,
            unread_count: row.get("nUnReadCount")?,
            session_nickname: row.get("session_str_nick_name")?,
            session_status: row.get("nStatus")?,
            is_send: row.get("nIsSend")?,
            content: row.get("strContent")?,
            msg_local_id: row.get("nMsgLocalID")?,
            msg_status: row.get("nMsgStatus")?,
            timestamp: timestamp_opt,
            time_str,
            msg_type: row.get("nMsgType")?,
            msg_sub_type: row.get("session_reserved2_msg_sub_type")?,
            contact_nickname: row.get("contact_nick_name")?,
            contact_remark: row.get("contact_remark")?,
            contact_account: row.get("contact_alias")?,
            contact_description: row.get("contact_reserved6_describe")?,
            contact_head_img_url: row.get("contact_big_head_img_url")?,
            contact_extra_buf_info,
            contact_label_list,
            contact_del_flag: row.get("contact_del_flag")?,
            contact_type: row.get("contact_type")?,
            contact_verify_flag: row.get("contact_verify_flag")?,
            contact_chat_room_type: row.get("contact_chat_room_type")?,
            contact_chat_room_notify: row.get("contact_chat_room_notify")?,
        })
    })?;

    let mut sessions = Vec::new();
    for row_result in mapped_rows {
        match row_result {
            Ok(session_info) => sessions.push(session_info),
            Err(e) => {
                // Log error and continue, to collect all successfully mapped ones
                eprintln!("Error processing a session row, skipping: {}", e);
            }
        }
    }

    Ok(sessions)
}
pub fn get_recent_chat_wxids(conn: &Connection, limit: usize) -> Result<Vec<String>, anyhow::Error> {
    let sql = "
        SELECT strUsrName
        FROM Session
        WHERE strUsrName NOT LIKE '%@chatroom'
          AND strUsrName NOT LIKE '%@openim'
          AND strUsrName NOT LIKE 'gh_%'
        ORDER BY nOrder DESC
        LIMIT ?;
    ";

    let mut stmt = conn.prepare(sql)?;
    let wxids_iter = stmt.query_map([limit], |row| {
        let wxid: String = row.get(0)?;
        Ok(wxid)
    })?;

    let mut wxids = Vec::new();
    for wxid_result in wxids_iter {
        wxids.push(wxid_result?);
    }

    Ok(wxids)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_extra_buf_empty_or_none() {
        assert!(parse_extra_buf(None).unwrap().is_none());
        assert!(parse_extra_buf(Some(&[])).unwrap().is_none());
    }

    #[test]
    fn test_parse_gender() {
        // 74752C06 (key) 01 (type_id=int) 01 (length=1) 01 (value=1, male)
        let hex_data = "74752C06010101";
        let bytes = hex::decode(hex_data).unwrap();
        let result = parse_extra_buf(Some(&bytes)).unwrap().unwrap();
        assert_eq!(result.gender, Some(1));
    }

    #[test]
    fn test_parse_signature_utf16() {
        // 46CF10C4 (key) 02 (type_id=string) 0A00 (length=10 bytes, 5 chars) 480065006C006C006F00 ("Hello" in UTF-16LE)
        let hex_data = "46CF10C4020A00480065006C006C006F00";
        let bytes = hex::decode(hex_data).unwrap();
        let result = parse_extra_buf(Some(&bytes)).unwrap().unwrap();
        assert_eq!(result.signature, Some("Hello".to_string()));
    }

    #[test]
    fn test_parse_multiple_fields() {
        // Gender: 1 (Male)
        let gender_hex = "74752C06010101";
        // Signature: "Test" (T e s t in UTF-16LE)
        // 5400650073007400
        let signature_hex = "46CF10C40208005400650073007400";
        // Country: "CN" (C N in UTF-16LE)
        // 43004E00
        let country_hex = "A4D9024A02040043004E00";

        let combined_hex = format!("{}{}{}", gender_hex, signature_hex, country_hex);
        let bytes = hex::decode(combined_hex).unwrap();
        let result = parse_extra_buf(Some(&bytes)).unwrap().unwrap();

        assert_eq!(result.gender, Some(1));
        assert_eq!(result.signature, Some("Test".to_string()));
        assert_eq!(result.country, Some("CN".to_string()));
        assert!(result.province.is_none()); // Province not in data
    }

     #[test]
    fn test_parse_real_world_example_shortened() {
        // This is a shortened and modified example based on typical ExtraBuf structure
        // Contains: Gender (Female=2), Signature ("Test Signature"), Country ("US")
        let hex_data = "someprefixbytes\
                        74752C06010102\
                        somerandombytesbetween\
                        46CF10C4021C00540065007300740020005300690067006E0061007400750072006500\
                        anotherbunchofrandombytes\
                        A4D9024A02040055005300\
                        suffixbytes";
        let bytes = hex::decode(hex_data).unwrap();
        let result = parse_extra_buf(Some(&bytes)).unwrap().unwrap();

        assert_eq!(result.gender, Some(2));
        assert_eq!(result.signature, Some("Test Signature".to_string()));
        assert_eq!(result.country, Some("US".to_string()));
        assert!(result.city.is_none());
    }


    #[test]
    fn test_parse_extra_buf_with_unknown_data_and_partial_match() {
        // Key for gender, but data is incomplete or malformed after key
        let hex_data_malformed_gender = "74752C0601"; // Missing length and value
        let bytes_malformed_gender = hex::decode(hex_data_malformed_gender).unwrap();
        let result_malformed_gender = parse_extra_buf(Some(&bytes_malformed_gender)).unwrap().unwrap();
        assert!(result_malformed_gender.gender.is_none());

        // Valid gender, then key for signature but incomplete data
        let hex_data_partial_sig = "74752C0601010146CF10C4020A"; // Signature key + type + partial length
        let bytes_partial_sig = hex::decode(hex_data_partial_sig).unwrap();
        let result_partial_sig = parse_extra_buf(Some(&bytes_partial_sig)).unwrap().unwrap();
        assert_eq!(result_partial_sig.gender, Some(1));
        assert!(result_partial_sig.signature.is_none());
    }
}