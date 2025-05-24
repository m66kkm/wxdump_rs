// src/core/offsets.rs

use anyhow::{Result, anyhow};
use serde_json::Value;
use std::{collections::HashMap, fs, path::Path};

pub type WxOffsets = HashMap<String, Vec<isize>>; // Version string to list of offsets

const WX_OFFS_FILE_NAME: &str = "WX_OFFS.json";

// Attempts to find WX_OFFS.json in common locations:
// 1. Current executable's directory
// 2. Parent directory of the current executable's directory (if executable is in a subfolder like target/debug)
// 3. Workspace root (if running with `cargo run` from workspace root)
fn find_wx_offs_json() -> Option<String> {
    let current_exe = std::env::current_exe().ok()?;
    let current_dir = current_exe.parent()?;

    // Check 1: Next to executable
    let path1 = current_dir.join(WX_OFFS_FILE_NAME);
    if path1.exists() {
        return Some(path1.to_string_lossy().into_owned());
    }

    // Check 2: Parent of executable's dir (e.g., target/debug -> target -> workspace root)
    if let Some(parent_dir) = current_dir.parent() {
        let path2 = parent_dir.join(WX_OFFS_FILE_NAME);
        if path2.exists() {
            return Some(path2.to_string_lossy().into_owned());
        }
        // Check 2.1: One more level up (e.g., target -> workspace root)
        if let Some(grandparent_dir) = parent_dir.parent() {
            let path2_1 = grandparent_dir.join(WX_OFFS_FILE_NAME);
            if path2_1.exists() {
                return Some(path2_1.to_string_lossy().into_owned());
            }
        }
    }
    
    // Check 3: Current working directory (CWD of the executable)
    if let Ok(cwd) = std::env::current_dir() {
        let path_in_cwd = cwd.join(WX_OFFS_FILE_NAME);
        if path_in_cwd.exists() {
            return Some(path_in_cwd.to_string_lossy().into_owned());
        }

        // Check 4: Relative to CWD, for finding original pysrc/pywxdump/WX_OFFS.json
        // This assumes CWD is the rust project root (e.g., wxdump_rust_core)
        // and WX_OFFS.json is in ../pysrc/pywxdump/
        let path_relative_to_pysrc = cwd.join("../pysrc/pywxdump/").join(WX_OFFS_FILE_NAME);
        if let Ok(canonical_path_relative) = path_relative_to_pysrc.canonicalize() {
            if canonical_path_relative.exists() {
                 return Some(canonical_path_relative.to_string_lossy().into_owned());
            }
        }
    }
    // Fallback: Original "Check 4" logic, in case CWD was actually the workspace root.
    // This is less likely with `cargo run` from package dir but kept as a fallback.
    let path_original_check4 = Path::new("pysrc/pywxdump/").join(WX_OFFS_FILE_NAME);
    if path_original_check4.exists() {
        if let Ok(canonical_path_original_check4) = path_original_check4.canonicalize() {
            if canonical_path_original_check4.exists() {
                return Some(canonical_path_original_check4.to_string_lossy().into_owned());
            }
        }
    }

    None
}


pub fn load_wx_offsets() -> Result<WxOffsets> {
    let wx_offs_path_str = find_wx_offs_json()
        .ok_or_else(|| anyhow!("{} not found in standard locations.", WX_OFFS_FILE_NAME))?;
    
    println!("[Offsets] Found {} at: {}", WX_OFFS_FILE_NAME, wx_offs_path_str);

    let file_content = fs::read_to_string(&wx_offs_path_str)
        .map_err(|e| anyhow!("Failed to read {}: {}", wx_offs_path_str, e))?;
    
    let parsed_json: Value = serde_json::from_str(&file_content)
        .map_err(|e| anyhow!("Failed to parse JSON from {}: {}", wx_offs_path_str, e))?;

    if let Value::Object(map) = parsed_json {
        let mut offsets_map: WxOffsets = HashMap::new();
        for (version, value) in map {
            if let Value::Array(arr) = value {
                let mut version_offsets: Vec<isize> = Vec::new();
                for val_num in arr {
                    if let Value::Number(num) = val_num {
                        // JSON numbers can be i64, u64, or f64. We expect integers for offsets.
                        if let Some(offset) = num.as_i64() {
                            version_offsets.push(offset as isize);
                        } else {
                            // Handle potential f64 if necessary, or error out
                            return Err(anyhow!("Non-integer offset found for version {}: {:?}", version, num));
                        }
                    } else {
                        return Err(anyhow!("Non-number value found in offset array for version {}: {:?}", version, val_num));
                    }
                }
                offsets_map.insert(version, version_offsets);
            } else {
                return Err(anyhow!("Value for version {} is not an array in {}", version, wx_offs_path_str));
            }
        }
        Ok(offsets_map)
    } else {
        Err(anyhow!("Root of {} is not a JSON object.", wx_offs_path_str))
    }
}