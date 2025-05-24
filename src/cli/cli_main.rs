use clap::Parser;
// Assuming cli.rs is in src/cli.rs and lib.rs has `pub mod cli;`
use wxdump_rs::cli::{Cli, Commands};
use wxdump_rs::core::db_parser::micro_msg_parser::{Contact, get_contacts, get_chat_rooms, ChatRoomInfo, get_sessions, SessionInfo, get_recent_chat_wxids};
use wxdump_rs::core::db_parser::connect_sqlite_db;

fn main() -> anyhow::Result<()> {
    let cli_args = Cli::parse();

    match cli_args.command {
        Commands::Bias { mobile, name, account, key, db_path, wx_offs_path } => {
            println!("Command: Bias");
            println!("CLI Args Received:");
            println!("  Mobile: {}", mobile);
            println!("  Name: {}", name);
            println!("  Account: {}", account);
            if let Some(k) = &key { // Borrowing key
                println!("  Key: {}", k);
            }
            if let Some(p) = &db_path { // Borrowing db_path
                println!("  DB Path: {:?}", p);
            }
            if let Some(p) = &wx_offs_path { // Borrowing wx_offs_path
                println!("  WX Offsets Path: {:?}", p);
            }
            println!("\nAttempting to extract WeChat info (simulating full bias logic)...");
            
            // 1. Load offsets (similar to main.rs, but wx_offs_path from CLI can override)
            // For now, we'll just use the default loading mechanism from core::offsets
            // In a real implementation, you'd check if wx_offs_path is Some and load from there.
            let loaded_offsets_map = match wxdump_rs::core::offsets::load_wx_offsets() {
                Ok(offsets) => {
                    println!("[Bias Command] Successfully loaded {} offset entries.", offsets.len());
                    offsets
                }
                Err(e) => {
                    eprintln!("[Bias Command] Error loading WX_OFFS.json: {}. Cannot proceed with info extraction.", e);
                    return Ok(()); // Or handle error differently
                }
            };

            // 2. Extract info
            match wxdump_rs::core::info_extractor::extract_all_wechat_info(&loaded_offsets_map) {
                Ok(user_infos) => {
                    if user_infos.is_empty() {
                        println!("[Bias Command] No WeChat user info extracted.");
                    }
                    for user_info_extracted in user_infos {
                        println!("[Bias Command] ---- Extracted Info for PID: {} ----", user_info_extracted.pid);
                        println!("  Version: {}", user_info_extracted.version);
                        println!("  Account (extracted): {}", user_info_extracted.account.as_deref().unwrap_or("N/A"));
                        println!("  Nickname (extracted): {}", user_info_extracted.nickname.as_deref().unwrap_or("N/A"));
                        println!("  Mobile (extracted): {}", user_info_extracted.mobile.as_deref().unwrap_or("N/A"));
                        println!("  Mail (extracted): {}", user_info_extracted.mail.as_deref().unwrap_or("N/A"));
                        println!("  Key (extracted): {}", user_info_extracted.key.as_deref().unwrap_or("N/A"));
                        println!("  WxID (extracted): {}", user_info_extracted.wxid.as_deref().unwrap_or("N/A"));
                        println!("  WeChat Files Path (extracted): {}", user_info_extracted.wx_files_path.as_ref().map_or_else(|| String::from("N/A"), |p_buf| p_buf.to_string_lossy().to_string()));
                        println!("  User DB Path (extracted): {}", user_info_extracted.wx_user_db_path.as_ref().map_or_else(|| String::from("N/A"), |p_buf| p_buf.to_string_lossy().to_string()));
                        // Here, you would compare/use the CLI args (mobile, name, account, key, db_path)
                        // with the extracted user_info_extracted.
                        // For now, we just print both.
                    }
                }
                Err(e) => {
                    eprintln!("[Bias Command] Error extracting WeChat info: {}", e);
                }
            }
        }
        Commands::Info { wx_offs_path, save_path } => {
            println!("Command: Info");
            if let Some(p) = wx_offs_path {
                println!("  WX Offsets Path: {:?}", p);
            }
            if let Some(p) = save_path {
                println!("  Save Path: {:?}", p);
            }
        }
        Commands::WxPath { db_types, wx_files, wxid } => {
            println!("Command: WxPath");
            if let Some(types) = db_types {
                println!("  DB Types: {}", types);
            }
            if let Some(p) = wx_files {
                println!("  WX Files Path: {:?}", p);
            }
            if let Some(id) = wxid {
                println!("  WxID: {}", id);
            }
        }
        Commands::Decrypt { key, db_path, out_path } => {
            println!("Command: Decrypt");
            println!("  Key: {}", key);
            println!("  DB Path: {:?}", db_path);
            println!("  Out Path: {:?}", out_path);
        }
        Commands::Merge { db_path, out_path } => {
            println!("Command: Merge");
            println!("  DB Path: {}", db_path); // This is a String of comma-separated paths
            println!("  Out Path: {:?}", out_path);
        }
        Commands::DbShow { merge_path, wx_path, my_wxid, online } => {
            println!("Command: DbShow");
            println!("  Merge Path: {:?}", merge_path);
            if let Some(p) = wx_path {
                println!("  WX Path: {:?}", p);
            }
            println!("  My WxID: {}", my_wxid);
            println!("  Online: {}", online);
        }
        Commands::TableDump { db_path, table_name } => {
            println!("Command: TableDump");
            println!("  DB Path: {:?}", db_path);
            println!("  Table Name: {}", table_name);

            let mut absolute_db_path = db_path.clone();
            if !absolute_db_path.is_absolute() {
                match std::env::current_dir() {
                    Ok(cwd) => {
                        absolute_db_path = cwd.join(absolute_db_path);
                        println!("Resolved relative DB path to: {:?}", absolute_db_path);
                    }
                    Err(e) => {
                        eprintln!("Failed to get current working directory: {}. Please use an absolute path for --db-path.", e);
                        return Ok(()); // Or handle error appropriately
                    }
                }
            }

            match wxdump_rs::core::db_parser::connect_sqlite_db(&absolute_db_path) {
                Ok(conn) => {
                    println!("Successfully connected to database: {:?}", absolute_db_path);
                    // 可选：列出所有表
                    // match wxdump_rust_core::core::db_parser::get_table_names(&conn) {
                    //     Ok(tables) => {
                    //         println!("Available tables: {:?}", tables);
                    //     }
                    //     Err(e) => {
                    //         eprintln!("Error listing tables: {}", e);
                    //     }
                    // }

                    match wxdump_rs::core::db_parser::get_all_rows_from_table(&conn, &table_name) {
                        Ok(rows) => {
                            if rows.is_empty() {
                                println!("Table '{}' is empty or does not exist.", table_name);
                            } else {
                                println!("First {} rows from table '{}':", std::cmp::min(5, rows.len()), table_name);
                                for (i, row_map) in rows.iter().take(5).enumerate() {
                                    print!("  Row {}: ", i + 1);
                                    let mut first_col = true;
                                    for (col_name, value) in row_map.iter().take(3) { // 只打印前3列以保持简洁
                                        if !first_col {
                                            print!(", ");
                                        }
                                        print!("{}: {:?}", col_name, value);
                                        first_col = false;
                                    }
                                    if row_map.len() > 3 {
                                        print!(", ..."); // 表示还有更多列
                                    }
                                    println!();
                                }
                                if rows.len() > 5 {
                                    println!("  ... and {} more rows.", rows.len() - 5);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error getting rows from table '{}': {}", table_name, e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error connecting to database '{:?}': {}", absolute_db_path, e);
                }
            }
        }
        Commands::ShowContacts { db_path, word, wxids, label_ids } => {
            println!("Command: ShowContacts");
            println!("  DB Path: {:?}", db_path);
            if let Some(w) = &word {
                println!("  Word: {}", w);
            }
            if let Some(ids) = &wxids {
                println!("  WxIDs: {:?}", ids);
            }
            if let Some(l_ids) = &label_ids {
                println!("  Label IDs: {:?}", l_ids);
            }

            let mut absolute_db_path = db_path.clone();
            if !absolute_db_path.is_absolute() {
                match std::env::current_dir() {
                    Ok(cwd) => {
                        absolute_db_path = cwd.join(absolute_db_path);
                        println!("Resolved relative DB path to: {:?}", absolute_db_path);
                    }
                    Err(e) => {
                        eprintln!("Failed to get current working directory: {}. Please use an absolute path for --db-path.", e);
                        return Ok(());
                    }
                }
            }

            match connect_sqlite_db(&absolute_db_path) {
                Ok(conn) => {
                    println!("Successfully connected to database: {:?}", absolute_db_path);
                    
                    // Convert Option<String> to Option<&str> and Option<Vec<String>> to Option<&[String]>
                    let word_ref = word.as_deref();
                    let wxids_ref = wxids.as_ref().map(|v| v.as_slice());
                    // label_ids is Option<Vec<i64>>, get_contacts expects Option<&[i64]>
                    let label_ids_ref = label_ids.as_ref().map(|v| v.as_slice());

                    match get_contacts(&conn, word_ref, wxids_ref, label_ids_ref) {
                        Ok(contacts) => {
                            if contacts.is_empty() {
                                println!("No contacts found matching the criteria.");
                            } else {
                                println!("Found {} contacts:", contacts.len());
                                for (i, contact) in contacts.iter().enumerate() {
                                    println!("--- Contact {} ---", i + 1);
                                    println!("  WxID: {}", contact.wxid); // wxid is String, not Option<String>
                                    println!("  Nickname: {}", contact.nickname.as_deref().unwrap_or("N/A"));
                                    println!("  Remark: {}", contact.remark.as_deref().unwrap_or("N/A"));
                                    println!("  Account: {}", contact.account.as_deref().unwrap_or("N/A"));
                                    if !contact.label_list.is_empty() { // label_list is Vec<String>, not Option
                                        println!("  Labels: {}", contact.label_list.join(", "));
                                    }
                                    if let Some(extra_info) = &contact.extra_buf_info {
                                        if let Some(gender) = extra_info.gender {
                                            println!("  Gender: {}", gender);
                                        }
                                        if let Some(country) = &extra_info.country {
                                            print!("  Region: {}", country);
                                            if let Some(province) = &extra_info.province {
                                                print!(", {}", province);
                                            }
                                            if let Some(city) = &extra_info.city {
                                                print!(", {}", city);
                                            }
                                            println!();
                                        }
                                    }
                                }
                                if contacts.len() > 10 {
                                     println!("... (output truncated, showing first 10 contacts)");
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error getting contacts: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error connecting to database '{:?}': {}", absolute_db_path, e);
                }
            }
        }
        Commands::ShowChatrooms { db_path, room_wxids } => {
            println!("Command: ShowChatrooms");
            println!("  DB Path: {:?}", db_path);
            if let Some(ids) = &room_wxids {
                println!("  Room WxIDs: {:?}", ids);
            }

            let mut absolute_db_path = db_path.clone();
            if !absolute_db_path.is_absolute() {
                match std::env::current_dir() {
                    Ok(cwd) => {
                        absolute_db_path = cwd.join(absolute_db_path);
                        println!("Resolved relative DB path to: {:?}", absolute_db_path);
                    }
                    Err(e) => {
                        eprintln!("Failed to get current working directory: {}. Please use an absolute path for --db-path.", e);
                        return Ok(());
                    }
                }
            }

            match connect_sqlite_db(&absolute_db_path) {
                Ok(conn) => {
                    println!("Successfully connected to database: {:?}", absolute_db_path);
                    match get_chat_rooms(&conn, room_wxids.as_ref().map(|v| v.as_slice())) {
                        Ok(chat_rooms) => {
                            if chat_rooms.is_empty() {
                                println!("No chat rooms found matching the criteria.");
                            } else {
                                println!("Found {} chat room(s):", chat_rooms.len());
                                for (wxid, room_info) in chat_rooms {
                                    println!("--- Chat Room: {} ---", wxid);
                                    println!("  Announcement: {}", room_info.announcement.as_deref().unwrap_or("N/A"));
                                    println!("  Owner WxID: {}", room_info.owner_wxid.as_deref().unwrap_or("N/A"));
                                    println!("  Member Count: {}", room_info.members.len());
                                    if !room_info.members.is_empty() {
                                        println!("  Members (showing up to 5):");
                                        for (i, member) in room_info.members.iter().take(5).enumerate() {
                                            println!("    {}. WxID: {}, Nickname: {}",
                                                     i + 1,
                                                     member.wxid,
                                                     member.room_nickname.as_deref().unwrap_or("N/A (parsing pending)"));
                                        }
                                        if room_info.members.len() > 5 {
                                            println!("    ... and {} more members.", room_info.members.len() - 5);
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error getting chat rooms: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error connecting to database '{:?}': {}", absolute_db_path, e);
                }
            }
        }
        Commands::ShowSessions { db_path, limit } => {
            println!("Command: ShowSessions");
            println!("  DB Path: {:?}", db_path);
            if let Some(l) = limit {
                println!("  Limit: {}", l);
            }

            let mut absolute_db_path = db_path.clone();
            if !absolute_db_path.is_absolute() {
                match std::env::current_dir() {
                    Ok(cwd) => {
                        absolute_db_path = cwd.join(absolute_db_path);
                        println!("Resolved relative DB path to: {:?}", absolute_db_path);
                    }
                    Err(e) => {
                        eprintln!("Failed to get current working directory: {}. Please use an absolute path for --db-path.", e);
                        return Ok(());
                    }
                }
            }

            match connect_sqlite_db(&absolute_db_path) {
                Ok(conn) => {
                    println!("Successfully connected to database: {:?}", absolute_db_path);
                    match get_sessions(&conn) {
                        Ok(mut sessions) => {
                            if let Some(l) = limit {
                                sessions.truncate(l);
                            }

                            if sessions.is_empty() {
                                println!("No sessions found.");
                            } else {
                                println!("Found {} session(s):", sessions.len());
                                for (i, session) in sessions.iter().enumerate() {
                                    println!("--- Session {} ---", i + 1);
                                    println!("  WxID: {}", session.wxid);
                                    let display_name = session.session_nickname
                                        .as_deref()
                                        .or(session.contact_remark.as_deref())
                                        .or(session.contact_nickname.as_deref())
                                        .unwrap_or("N/A");
                                    println!("  Nickname: {}", display_name);
                                    println!("  Latest Message: {}", session.content.as_deref().unwrap_or("N/A"));
                                    println!("  Time: {}", session.time_str.as_deref().unwrap_or("N/A"));
                                    println!("  Unread Count: {}", session.unread_count.map_or_else(|| 0.to_string(), |c| c.to_string()));
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error getting sessions: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error connecting to database '{:?}': {}", absolute_db_path, e);
                }
            }
        }
        Commands::ShowRecentWxids { db_path, limit } => {
            println!("Command: ShowRecentWxids");
            println!("  DB Path: {:?}", db_path);
            println!("  Limit: {}", limit);

            let mut absolute_db_path = db_path.clone();
            if !absolute_db_path.is_absolute() {
                match std::env::current_dir() {
                    Ok(cwd) => {
                        absolute_db_path = cwd.join(absolute_db_path);
                        println!("Resolved relative DB path to: {:?}", absolute_db_path);
                    }
                    Err(e) => {
                        eprintln!("Failed to get current working directory: {}. Please use an absolute path for --db-path.", e);
                        return Ok(());
                    }
                }
            }

            match connect_sqlite_db(&absolute_db_path) {
                Ok(conn) => {
                    println!("Successfully connected to database: {:?}", absolute_db_path);
                    match get_recent_chat_wxids(&conn, limit) {
                        Ok(wxids) => {
                            if wxids.is_empty() {
                                println!("No recent chat wxids found.");
                            } else {
                                println!("Found {} recent chat wxid(s):", wxids.len());
                                for (i, wxid) in wxids.iter().enumerate() {
                                    println!("  {}. {}", i + 1, wxid);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error getting recent chat wxids: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error connecting to database '{:?}': {}", absolute_db_path, e);
                }
            }
        }
        // The Ui and Api commands are commented out in cli.rs, so no need to handle them here
        // unless they are uncommented.
        // _ => {
        //     // This should not be reached if all commands are handled
        //     eprintln!("Unhandled command variant.");
        // }
    }

    Ok(())
}