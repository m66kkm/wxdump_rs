// src/core/info_extractor.rs

use anyhow::{Result, anyhow};
use std::path::PathBuf;
use super::win_api::{self}; // Removed unused ProcessInfo
use super::offsets::WxOffsets;

#[derive(Debug, Clone, Default)]
pub struct WeChatUserInfo {
    pub pid: u32,
    pub version: String,
    pub account: Option<String>,
    pub mobile: Option<String>,
    pub nickname: Option<String>,
    pub mail: Option<String>,
    pub wxid: Option<String>,
    pub key: Option<String>,
    pub wx_files_path: Option<PathBuf>, 
    pub wx_user_db_path: Option<PathBuf>, 
}

fn get_wechat_files_path_from_registry() -> Result<Option<PathBuf>> {
    const WECHAT_REG_KEY_PATH: &str = "Software\\Tencent\\WeChat";
    const WECHAT_FILES_VALUE_NAME: &str = "FileSavePath";

    match win_api::read_registry_sz_value(
        windows_sys::Win32::System::Registry::HKEY_CURRENT_USER,
        WECHAT_REG_KEY_PATH,
        WECHAT_FILES_VALUE_NAME,
    ) {
        Ok(path_str) => {
            if path_str == "MyDocument:" { 
                if let Some(user_profile) = std::env::var("USERPROFILE").ok() {
                    let docs_path = PathBuf::from(user_profile).join("Documents");
                    let wechat_files_path = docs_path.join("WeChat Files");
                    if wechat_files_path.exists() && wechat_files_path.is_dir(){
                        println!("[InfoExtractor] Resolved 'MyDocument:' to WeChat Files path: {:?}", wechat_files_path);
                        return Ok(Some(wechat_files_path));
                    } else {
                         println!("[InfoExtractor] 'MyDocument:' resolved path does not exist or not a dir: {:?}", wechat_files_path);
                        return Ok(None);
                    }
                } else {
                     println!("[InfoExtractor] Could not resolve 'MyDocument:' due to missing USERPROFILE.");
                    return Ok(None);
                }
            } else if !path_str.is_empty() {
                let path_str_clone_for_join = path_str.clone(); // Clone for the first PathBuf creation
                let wechat_files_path = PathBuf::from(path_str_clone_for_join).join("WeChat Files"); 
                 if wechat_files_path.exists() && wechat_files_path.is_dir(){
                    println!("[InfoExtractor] Found WeChat Files path from registry (joined): {:?}", wechat_files_path);
                    return Ok(Some(wechat_files_path));
                } else {
                    let original_path_buf = PathBuf::from(&path_str); // Borrow original path_str
                    if original_path_buf.exists() && original_path_buf.is_dir() && original_path_buf.file_name().map_or(false, |name| name == "WeChat Files") {
                        println!("[InfoExtractor] Found WeChat Files path from registry (original path): {:?}", original_path_buf);
                        return Ok(Some(original_path_buf));
                    }
                    println!("[InfoExtractor] Registry path for WeChat Files does not exist or not a dir: {:?} (and original path {:?} also invalid)", wechat_files_path, path_str);
                    return Ok(None);
                }
            }
            Ok(None)
        }
        Err(e) => {
            println!("[InfoExtractor] Failed to read WeChat FileSavePath from registry: {}. This might be normal.", e);
            Ok(None)
        }
    }
}

fn get_wechat_files_path_from_memory(pid: u32, wxid: &str) -> Result<Option<PathBuf>> {
    if wxid.is_empty() {
        return Ok(None);
    }
    let wxid_bytes = wxid.as_bytes();
    let search_start_address = 0x0;
    let search_end_address = usize::MAX; 

    match win_api::search_memory_for_pattern(pid, wxid_bytes, search_start_address, search_end_address, 5) { 
        Ok(addresses) => {
            if addresses.is_empty() {
                println!("[InfoExtractor] WxID pattern for path search not found in memory for PID {}.", pid);
                return Ok(None);
            }
            for &addr in &addresses {
                let read_len = 260; 
                if addr < 100 { continue; } 
                let read_start_addr = addr - 100; 
                if let Ok(buffer) = win_api::read_process_memory(pid, read_start_addr, read_len) {
                    for i in 0..buffer.len() {
                        if i + 2 < buffer.len() && buffer[i].is_ascii_alphabetic() && buffer[i+1] == b':' && buffer[i+2] == b'\\' {
                            let potential_path_bytes_vec: Vec<u8> = buffer[i..].iter().take_while(|&&b| b != 0).cloned().collect();
                            if let Ok(path_str) = String::from_utf8(potential_path_bytes_vec) {
                                if path_str.contains("WeChat Files") && path_str.contains(wxid) {
                                    if let Some(wc_files_end_idx) = path_str.find("WeChat Files") {
                                        let root_path_str = &path_str[..(wc_files_end_idx + "WeChat Files".len())];
                                        let path_buf = PathBuf::from(root_path_str);
                                        if path_buf.exists() && path_buf.is_dir() {
                                            println!("[InfoExtractor] Found potential WeChat Files path via memory search: {:?}", path_buf);
                                            return Ok(Some(path_buf));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Ok(None) 
        }
        Err(e) => {
            eprintln!("[InfoExtractor] Error searching memory for WxID pattern (for path): {}", e);
            Ok(None)
        }
    }
}

fn get_key_from_memory_search(pid: u32, pointer_size: usize) -> Result<Option<String>> {
    println!("[InfoExtractor DEBUG] Attempting memory search for key using anchor strings (Python-like).");
    let wechat_win_dll_base = match win_api::get_module_base_address(pid, "WeChatWin.dll") {
        Ok(addr) => addr,
        Err(e) => { eprintln!("[InfoExtractor DEBUG] WeChatWin.dll not found for key search: {}", e); return Ok(None); }
    };
    let search_start_address = wechat_win_dll_base;
    let search_end_address = wechat_win_dll_base.saturating_add(100 * 1024 * 1024); 
    const KEY_LEN: usize = 32;
    let anchor_strings: [&[u8]; 3] = [b"iphone\x00", b"android\x00", b"ipad\x00"];
    let mut found_anchor_addrs = Vec::new();

    for anchor in &anchor_strings {
        match win_api::search_memory_for_pattern(pid, anchor, search_start_address, search_end_address, 5) {
            Ok(addrs) => {
                if !addrs.is_empty() {
                    println!("[InfoExtractor DEBUG] Found anchor {:?} at addresses: {:?}", String::from_utf8_lossy(anchor), addrs.iter().map(|a| format!("0x{:X}", a)).collect::<Vec<_>>());
                    found_anchor_addrs.extend_from_slice(&addrs);
                }
            }
            Err(e) => { println!("[InfoExtractor DEBUG] Error searching for anchor {:?}: {}", String::from_utf8_lossy(anchor), e); }
        }
    }
    if found_anchor_addrs.is_empty() {
        println!("[InfoExtractor DEBUG] No anchor strings found for Python-like key search in WeChatWin.dll range.");
        return Ok(None);
    }
    found_anchor_addrs.sort_unstable();
    found_anchor_addrs.dedup();

    for &anchor_addr in found_anchor_addrs.iter().rev() {
        let scan_start_iteration = anchor_addr; 
        let scan_end_iteration = anchor_addr.saturating_sub(2000);
        for ptr_addr_to_check in (scan_end_iteration..=scan_start_iteration).rev().step_by(pointer_size) {
            if ptr_addr_to_check < search_start_address || ptr_addr_to_check.saturating_add(pointer_size) > search_end_address { continue; }
            match win_api::read_process_memory(pid, ptr_addr_to_check, pointer_size) {
                Ok(ptr_bytes) => {
                    if ptr_bytes.len() == pointer_size {
                        let key_address = if pointer_size == 8 { u64::from_le_bytes(ptr_bytes.try_into().unwrap_or_default()) as usize } 
                                          else { u32::from_le_bytes(ptr_bytes.try_into().unwrap_or_default()) as usize };
                        if key_address < 0x10000 { continue; }
                        if let Ok(key_bytes) = win_api::read_process_memory(pid, key_address, KEY_LEN) {
                            if key_bytes.len() == KEY_LEN && !key_bytes.iter().all(|&b| b == 0) {
                                let key_hex = hex::encode(&key_bytes);
                                println!("[InfoExtractor DEBUG] Python-like memory search found potential key at 0x{:X} (ptr at 0x{:X}): {}", key_address, ptr_addr_to_check, key_hex);
                                return Ok(Some(key_hex));
                            }
                        }
                    }
                }
                Err(_e) => { /* Silently continue */ }
            }
        }
    }
    println!("[InfoExtractor DEBUG] No key found via Python-like memory search after checking all anchors.");
    Ok(None)
}

pub fn extract_all_wechat_info(loaded_offsets: &WxOffsets) -> Result<Vec<WeChatUserInfo>> {
    let mut all_user_info = Vec::new();
    let processes = win_api::list_processes()?;

    for process in processes {
        if process.name == "WeChat.exe" {
            println!("[InfoExtractor] Found WeChat.exe with PID: {}", process.pid);
            let exe_path = match win_api::get_process_exe_path(process.pid) {
                Ok(p) => p,
                Err(e) => { eprintln!("[InfoExtractor] Failed to get exe path for PID {}: {}", process.pid, e); continue; }
            };
            let version = match win_api::get_file_version_info(&exe_path) {
                Ok(v) => v,
                Err(e) => { eprintln!("[InfoExtractor] Failed to get version for PID {} (path: {}): {}", process.pid, exe_path, e); "unknown".to_string() }
            };
            println!("[InfoExtractor] PID: {}, Path: {}, Version: {}", process.pid, exe_path, version);

            let mut user_info = WeChatUserInfo { pid: process.pid, version: version.clone(), ..Default::default() };
            let mut dll_base_address_opt: Option<usize> = None;
            let mut pointer_size_opt: Option<usize> = None;

            if let Some(v_offsets) = loaded_offsets.get(&version) {
                println!("[InfoExtractor] Found offsets for version {}: {:?}", version, v_offsets);
                if let Ok(arch_size) = win_api::get_process_architecture(process.pid) {
                    pointer_size_opt = Some(arch_size);
                    if let Ok(base_addr) = win_api::get_module_base_address(process.pid, "WeChatWin.dll") {
                        dll_base_address_opt = Some(base_addr);
                        println!("[InfoExtractor] WeChatWin.dll base: 0x{:X}, ArchSize: {}", base_addr, arch_size);

                        // Nickname, Account, Mobile, Mail
                        if v_offsets.len() > 0 && v_offsets[0] != 0 {
                            match read_string_via_pointer_offset(process.pid, base_addr, v_offsets[0], arch_size, 64) {
                                Ok(name) => { println!("[InfoExtractor] Nickname (ptr): {}", name); user_info.nickname = Some(name); },
                                Err(_e_ptr) => match read_direct_string_from_offset(process.pid, base_addr, v_offsets[0], 64) {
                                    Ok(name_direct) => { println!("[InfoExtractor] Nickname (direct): {}", name_direct); user_info.nickname = Some(name_direct); },
                                    Err(_e_direct) => eprintln!("[InfoExtractor] Failed to read nickname (ptr/direct)."),
                                }
                            }
                        }
                        if v_offsets.len() > 1 && v_offsets[1] != 0 {
                            match read_direct_string_from_offset(process.pid, base_addr, v_offsets[1], 32) {
                                Ok(acc) => { println!("[InfoExtractor] Account: {}", acc); user_info.account = Some(acc); },
                                Err(e) => eprintln!("[InfoExtractor] Failed to read account: {}", e),
                            }
                        }
                        if v_offsets.len() > 2 && v_offsets[2] != 0 {
                            match read_direct_string_from_offset(process.pid, base_addr, v_offsets[2], 64) {
                                Ok(mob) => { println!("[InfoExtractor] Mobile: {}", mob); user_info.mobile = Some(mob); },
                                Err(e) => eprintln!("[InfoExtractor] Failed to read mobile: {}", e),
                            }
                        }
                        if v_offsets.len() > 3 && v_offsets[3] != 0 {
                            match read_direct_string_from_offset(process.pid, base_addr, v_offsets[3], 64) {
                                Ok(em) => { println!("[InfoExtractor] Mail: {}", em); user_info.mail = Some(em); },
                                Err(e) => eprintln!("[InfoExtractor] Failed to read mail: {}", e),
                            }
                        }
                    } else { eprintln!("[InfoExtractor] Failed to get WeChatWin.dll base for PID {}.", process.pid); }
                } else { eprintln!("[InfoExtractor] Failed to get arch size for PID {}.", process.pid); }
            } else { println!("[InfoExtractor] No offsets for version {}.", version); }

            match get_wxid_from_memory(process.pid) {
                Ok(Some(wxid_val)) => { println!("[InfoExtractor] WxID (mem): {}", wxid_val); user_info.wxid = Some(wxid_val); },
                _ => println!("[InfoExtractor] WxID not found via memory search."),
            }

            let mut memory_search_attempted_for_path = false;
            match get_wechat_files_path_from_registry() {
                Ok(Some(reg_path)) => {
                    println!("[InfoExtractor] Path (reg): {:?}", reg_path);
                    user_info.wx_files_path = Some(reg_path.clone());
                    if let Some(id) = &user_info.wxid { user_info.wx_user_db_path = Some(reg_path.join(id)); }
                }
                _ => { // Ok(None) or Err
                    println!("[InfoExtractor] Path not in registry or error. Trying memory.");
                    memory_search_attempted_for_path = true;
                    if let Some(id) = &user_info.wxid {
                        match get_wechat_files_path_from_memory(process.pid, id) {
                            Ok(Some(mem_path)) => {
                                println!("[InfoExtractor] Path (mem): {:?}", mem_path);
                                user_info.wx_files_path = Some(mem_path.clone());
                                user_info.wx_user_db_path = Some(mem_path.join(id));
                            }
                            _ => println!("[InfoExtractor] Path not found via memory search."),
                        }
                    } else { println!("[InfoExtractor] No WxID to search path in memory."); }
                }
            }
            if !memory_search_attempted_for_path && user_info.wx_user_db_path.as_ref().map_or(true, |p| !p.exists()) {
                 println!("[InfoExtractor] Registry path for user DB invalid or not found. Trying memory for path.");
                 if let Some(id) = &user_info.wxid {
                        match get_wechat_files_path_from_memory(process.pid, id) {
                            Ok(Some(mem_path)) => {
                                println!("[InfoExtractor] Path (mem fallback): {:?}", mem_path);
                                user_info.wx_files_path = Some(mem_path.clone());
                                user_info.wx_user_db_path = Some(mem_path.join(id));
                            }
                            _ => println!("[InfoExtractor] Path not found via memory search (fallback)."),
                        }
                    } else { println!("[InfoExtractor] No WxID to search path in memory (fallback)."); }
            }


            if user_info.wx_user_db_path.is_some() {
                 println!("[InfoExtractor] User DB Path: {:?}", user_info.wx_user_db_path.as_ref().unwrap());
            } else {
                 println!("[InfoExtractor] User DB Path could not be determined.");
            }
            
            let mut key_from_offset_method: Option<String> = None;
            let mut key_from_memory_search_method: Option<String> = None;
            let expected_key_str = "ef135b887201452c9301f7ff774d83ce34852ab7f68844bfaae485b233626fe6";

            if let (Some(base_addr), Some(ptr_size)) = (dll_base_address_opt, pointer_size_opt) {
                if let Some(v_offsets) = loaded_offsets.get(&version) {
                    if v_offsets.len() > 4 && v_offsets[4] != 0 {
                        match read_key_via_pointer_offset(process.pid, base_addr, v_offsets[4], ptr_size) {
                            Ok(k) => { println!("[InfoExtractor] Key (offset): {}", k); key_from_offset_method = Some(k); },
                            Err(e) => eprintln!("[InfoExtractor] Failed key (offset): {}", e),
                        }
                    } else { println!("[InfoExtractor] Key offset invalid or 0."); }
                } else { println!("[InfoExtractor] No offsets for key.");}
                
                match get_key_from_memory_search(process.pid, ptr_size) {
                    Ok(Some(mk)) => { println!("[InfoExtractor] Key (mem): {}", mk); key_from_memory_search_method = Some(mk); },
                    _ => println!("[InfoExtractor] Key not found (mem)."),
                }
            } else { println!("[InfoExtractor] No DLL base/ptr size for key methods."); }

            let mut final_key_source = "None";
            if let Some(mem_k) = &key_from_memory_search_method {
                if mem_k == expected_key_str {
                    user_info.key = Some(mem_k.clone()); final_key_source = "Memory (Matches Expected)";
                } else {
                    if let Some(offset_k) = &key_from_offset_method {
                        if offset_k == expected_key_str {
                            user_info.key = Some(offset_k.clone()); final_key_source = "Offset (Matches Expected)";
                        } else { user_info.key = Some(mem_k.clone()); final_key_source = "Memory (No Match, Fallback)"; }
                    } else { user_info.key = Some(mem_k.clone()); final_key_source = "Memory (No Match, Offset Missing)"; }
                }
            } else if let Some(offset_k) = &key_from_offset_method {
                if offset_k == expected_key_str {
                    user_info.key = Some(offset_k.clone()); final_key_source = "Offset (Matches Expected, Mem Failed)";
                } else { user_info.key = Some(offset_k.clone()); final_key_source = "Offset (No Match, Mem Failed)"; }
            }
            println!("[InfoExtractor] Final key for PID {}: {:?} (Source: {})", process.pid, user_info.key, final_key_source);
            
            all_user_info.push(user_info);
        } 
    } 
    if all_user_info.is_empty() { println!("[InfoExtractor] No WeChat.exe processes found."); }
    Ok(all_user_info)
}

fn read_direct_string_from_offset(pid: u32, dll_base_address: usize, offset: isize, max_len: usize) -> Result<String> {
    if offset == 0 { return Err(anyhow!("Offset is zero.")); }
    let target_address = (dll_base_address as isize + offset) as usize;
    let bytes = win_api::read_process_memory(pid, target_address, max_len)?;
    let null_pos = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    if null_pos == 0 && bytes.is_empty() { return Ok("".to_string()); }
    String::from_utf8(bytes[..null_pos].to_vec()).map_err(|e| anyhow!("UTF-8 err from 0x{:X}: {}", target_address, e))
}

fn read_string_via_pointer_offset(pid: u32, dll_base_address: usize, offset: isize, pointer_size: usize, max_str_len: usize) -> Result<String> {
    if offset == 0 { return Err(anyhow!("Offset for pointer is zero.")); }
    let pointer_address = (dll_base_address as isize + offset) as usize;
    let pointer_bytes = win_api::read_process_memory(pid, pointer_address, pointer_size)?;
    if pointer_bytes.len() < pointer_size { return Err(anyhow!("Read too few bytes for ptr @ 0x{:X}", pointer_address)); }

    let string_address = match pointer_size {
        4 => u32::from_le_bytes(pointer_bytes.as_slice().try_into().unwrap()) as usize,
        8 => u64::from_le_bytes(pointer_bytes.as_slice().try_into().unwrap()) as usize,
        _ => return Err(anyhow!("Unsupported pointer size: {}", pointer_size)),
    };
    if string_address == 0 { return Err(anyhow!("Ptr @ 0x{:X} is null.", pointer_address)); }
    if string_address < 0x10000 { return Err(anyhow!("Ptr @ 0x{:X} -> low addr 0x{:X}.", pointer_address, string_address)); }

    let string_bytes = win_api::read_process_memory(pid, string_address, max_str_len)?;
    let null_pos = string_bytes.iter().position(|&b| b == 0).unwrap_or(string_bytes.len());
    if null_pos == 0 && string_bytes.is_empty() { return Ok("".to_string()); }
    String::from_utf8(string_bytes[..null_pos].to_vec()).map_err(|e| anyhow!("UTF-8 err from pointed addr 0x{:X}: {}", string_address, e))
}

fn read_key_via_pointer_offset(pid: u32, dll_base_address: usize, offset: isize, pointer_size: usize) -> Result<String> { 
    if offset == 0 { return Err(anyhow!("Offset for key pointer is zero.")); }
    let pointer_address = (dll_base_address as isize + offset) as usize;
    let pointer_bytes = win_api::read_process_memory(pid, pointer_address, pointer_size)?;
    if pointer_bytes.len() < pointer_size { return Err(anyhow!("Read too few bytes for key ptr @ 0x{:X}", pointer_address)); }

    let key_address = match pointer_size {
        4 => u32::from_le_bytes(pointer_bytes.as_slice().try_into().map_err(|_| anyhow!("Bytes to u32 key ptr"))?) as usize,
        8 => u64::from_le_bytes(pointer_bytes.as_slice().try_into().map_err(|_| anyhow!("Bytes to u64 key ptr"))?) as usize,
        _ => return Err(anyhow!("Unsupported pointer size for key: {}", pointer_size)),
    };
    if key_address < 0x10000 { return Err(anyhow!("Key ptr @ 0x{:X} -> low addr 0x{:X}.", pointer_address, key_address)); }
    if key_address == 0 { return Err(anyhow!("Key ptr @ 0x{:X} is null.", pointer_address)); }

    const KEY_LEN: usize = 32;
    let key_bytes = win_api::read_process_memory(pid, key_address, KEY_LEN)?;
    if key_bytes.len() < KEY_LEN { return Err(anyhow!("Read too few bytes for key @ 0x{:X}", key_address)); }
    Ok(hex::encode(key_bytes))
}

fn get_wxid_from_memory(pid: u32) -> Result<Option<String>> {
    let pattern_to_find = b"\\Msg\\FTSContact";
    let search_start_address = 0x0;
    let search_end_address = usize::MAX;

    match win_api::search_memory_for_pattern(pid, pattern_to_find, search_start_address, search_end_address, 100) {
        Ok(addresses) => {
            if addresses.is_empty() { return Ok(None); }
            let mut potential_wxids = Vec::new();
            for &pattern_start_addr in &addresses {
                if pattern_start_addr < 30 { continue; }
                let read_addr = pattern_start_addr - 30;
                if let Ok(buffer) = win_api::read_process_memory(pid, read_addr, 80) {
                    let mut split_before_msg = &buffer[..];
                    if let Some(msg_idx) = buffer.windows(4).position(|w| w == b"\\Msg") { split_before_msg = &buffer[..msg_idx]; }
                    if let Some(last_seg) = split_before_msg.rsplit(|&b| b == b'\\').next() {
                        if last_seg.starts_with(b"wxid_") {
                            if let Ok(s) = String::from_utf8(last_seg.to_vec()) { potential_wxids.push(s); }
                        }
                    }
                }
            }
            if !potential_wxids.is_empty() {
                let mut counts = std::collections::HashMap::new();
                potential_wxids.into_iter().for_each(|s| *counts.entry(s).or_insert(0) += 1);
                if let Some((id, _)) = counts.into_iter().max_by_key(|&(_, c)| c) { return Ok(Some(id)); }
            }
            Ok(None)
        }
        Err(e) => { eprintln!("[InfoExtractor:get_wxid] Error: {}", e); Ok(None) }
    }
}