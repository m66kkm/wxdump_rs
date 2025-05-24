mod core;

use std::path::PathBuf;
use std::fs;
use std::env; // For current_dir
use anyhow::anyhow; 
use walkdir::WalkDir; // Added for recursive directory traversal

fn main() -> anyhow::Result<()> {
    // println!("Attempting to load WX_OFFS.json...");
    // let loaded_offsets_map = match core::offsets::load_wx_offsets() {
    //     Ok(offsets) => {
    //         println!("[Main] Successfully loaded {} offset entries.", offsets.len());
    //         offsets
    //     }
    //     Err(e) => {
    //         eprintln!("[Main] Error loading WX_OFFS.json: {}", e);
    //         return Ok(());
    //     }
    // };

    // let project_root = env::current_dir().map_err(|e| anyhow!("Failed to get current directory: {}", e))?;
    // let base_temp_dir = project_root.join("temp"); // This will be wxdump_rust_core/temp
    
    // No longer creating a sub "msg" directory inside temp immediately, 
    // will create subdirs based on relative paths from the source Msg directory.

    // println!("\nExtracting WeChat User Info...");
    // match core::info_extractor::extract_all_wechat_info(&loaded_offsets_map) {
    //     Ok(user_infos) => {
    //         if user_infos.is_empty() {
    //             println!("[Main] No WeChat user info extracted.");
    //         }
    //         for user_info in user_infos {
    //             println!("[Main] ---- User Info for PID: {} ----", user_info.pid);
    //             println!("  Version: {}", user_info.version);
    //             println!("  Account: {}", user_info.account.as_deref().unwrap_or("N/A"));
    //             println!("  Nickname: {}", user_info.nickname.as_deref().unwrap_or("N/A"));
    //             // ... (other user info printouts)
    //             println!("  Key: {}", user_info.key.as_deref().unwrap_or("N/A"));
    //             println!("  User DB Path: {}", user_info.wx_user_db_path.as_ref().map_or_else(|| String::from("N/A"), |p_buf| p_buf.to_string_lossy().to_string()));
                
    //             if let Some(user_data_root_path) = &user_info.wx_user_db_path {
    //                 let source_msg_dir = user_data_root_path.join("Msg");
    //                 println!("[Main] Looking for files to decrypt in: {:?}", source_msg_dir);

    //                 if let Some(key_to_use_for_dbs) = &user_info.key {
    //                     if !source_msg_dir.exists() || !source_msg_dir.is_dir() {
    //                         println!("[Main] Source Msg directory not found at: {:?}", source_msg_dir);
    //                     } else {
    //                         println!("[Main] Starting recursive decryption for files in {:?}", source_msg_dir);
    //                         let mut decrypted_any_file = false;

    //                         for entry_result in WalkDir::new(&source_msg_dir).into_iter().filter_map(Result::ok) {
    //                             let entry_path = entry_result.path();
    //                             if entry_path.is_file() {
    //                                 // Calculate relative path from source_msg_dir
    //                                 let relative_path = entry_path.strip_prefix(&source_msg_dir)
    //                                     .map_err(|e| anyhow!("Failed to strip prefix for {:?}: {}", entry_path, e))?;
                                    
    //                                 // Construct target path in base_temp_dir, preserving relative structure
    //                                 // and prefixing filename with "de_"
    //                                 let target_parent_dir = if let Some(parent) = relative_path.parent() {
    //                                     base_temp_dir.join(parent)
    //                                 } else {
    //                                     base_temp_dir.clone() 
    //                                 };
                                    
    //                                 if !target_parent_dir.exists() {
    //                                     fs::create_dir_all(&target_parent_dir).map_err(|e| 
    //                                         anyhow!("Failed to create target parent directory {:?}: {}", target_parent_dir, e))?;
    //                                 }

    //                                 let original_filename = entry_path.file_name().ok_or_else(|| anyhow!("Failed to get filename for {:?}", entry_path))?;
    //                                 let decrypted_filename_str = format!("de_{}", original_filename.to_string_lossy());
    //                                 let decrypted_file_target_path = target_parent_dir.join(decrypted_filename_str);

    //                                 println!("[Main] Attempting to decrypt {:?} to {:?}", entry_path, decrypted_file_target_path);

    //                                 match core::decryption::decrypt_database_file(entry_path, &decrypted_file_target_path, key_to_use_for_dbs) {
    //                                     Ok(_) => {
    //                                         println!("[Main] Successfully decrypted {:?} to {:?}", entry_path.file_name().unwrap_or_default(), decrypted_file_target_path);
    //                                         decrypted_any_file = true;
    //                                         // DO NOT open or delete the file as per new instructions
    //                                     }
    //                                     Err(e) => {
    //                                         // Log decryption errors, but continue with other files
    //                                         if entry_path.extension().map_or(false, |ext| ext == "db" || ext == "sqlite") { // Only log for likely DBs
    //                                             eprintln!("[Main] Failed to decrypt potential DB file {:?}: {:?}", entry_path.file_name().unwrap_or_default(), e);
    //                                         } else {
    //                                             // For non-db files, decryption might fail expectedly (e.g. not encrypted, wrong format)
    //                                             // println!("[Main] Skipped non-DB or failed decryption for {:?}: {:?}", entry_path.file_name().unwrap_or_default(), e);
    //                                         }
    //                                     }
    //                                 }
    //                             }
    //                         }
    //                         if decrypted_any_file {
    //                             println!("[Main] Finished recursive decryption. Check {:?} for output.", base_temp_dir);
    //                         } else {
    //                             println!("[Main] No files were successfully decrypted from {:?}.", source_msg_dir);
    //                         }
    //                     }
    //                 } else {
    //                     println!("[Main] No key available. Cannot proceed with decryption for PID {}.", user_info.pid);
    //                 }
    //             } else {
    //                 println!("[Main] Cannot attempt decryption for PID {}: Missing DB user data path.", user_info.pid);
    //             }
    //             println!("[Main] ------------------------------");
    //         }
    //     }
    //     Err(e) => {
    //         eprintln!("[Main] Error extracting WeChat info: {}", e);
    //     }
    // }
    Ok(())
}
