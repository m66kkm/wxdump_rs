use log::warn;
use rusqlite::Connection;
use std::fs::{self};
use std::path::{Path, PathBuf};

use crate::wx_core::decryption::decrypt;
use crate::wx_core::utils::{wx_core_error, WxCoreError, WxCoreResult};

/// Merge multiple WeChat databases into a single database
pub fn merge_db(db_paths: &str, out_path: impl AsRef<Path>) -> WxCoreResult<PathBuf> {
    wx_core_error(|| {
        let out_path = out_path.as_ref();
        
        // Parse db_paths
        let db_paths: Vec<&str> = db_paths.split(',').map(|s| s.trim()).collect();
        
        if db_paths.is_empty() {
            return Err(WxCoreError::InvalidPath("No database paths provided".to_string()));
        }
        
        // Check if out_path is a directory or a file
        let out_file = if out_path.is_dir() {
            out_path.join("merge_all.db")
        } else {
            out_path.to_path_buf()
        };
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = out_file.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        // TODO: Implement the actual logic to merge databases
        // This would involve:
        // 1. Creating a new database
        // 2. Copying the schema from one of the source databases
        // 3. Copying the data from all source databases
        
        // For now, we'll just create an empty database
        let conn = Connection::open(&out_file)?;
        
        // Create a simple table to indicate this is a merged database
        conn.execute(
            "CREATE TABLE IF NOT EXISTS merged_info (
                id INTEGER PRIMARY KEY,
                source_path TEXT,
                merge_time TEXT
            )",
            [],
        )?;
        
        // Insert a record for each source database
        for db_path in db_paths {
            conn.execute(
                "INSERT INTO merged_info (source_path, merge_time) VALUES (?, datetime('now'))",
                [db_path],
            )?;
        }
        
        Ok(out_file)
    })
}

/// Decrypt and merge multiple WeChat databases
pub fn decrypt_merge(
    key: &str,
    db_paths: &[PathBuf],
    out_path: impl AsRef<Path>,
) -> WxCoreResult<PathBuf> {
    wx_core_error(|| {
        let out_path = out_path.as_ref();
        
        // Create a temporary directory for decrypted databases
        let temp_dir = out_path.join("temp_decrypt");
        if !temp_dir.exists() {
            fs::create_dir_all(&temp_dir)?;
        }
        
        // Decrypt each database
        let mut decrypted_paths = Vec::new();
        for db_path in db_paths {
            let file_name = db_path.file_name().ok_or_else(|| {
                WxCoreError::InvalidPath(format!("Invalid file name: {}", db_path.display()))
            })?;
            
            let out_file = temp_dir.join(format!("de_{}", file_name.to_string_lossy()));
            match decrypt(key, db_path, &out_file) {
                Ok(_) => decrypted_paths.push(out_file),
                Err(e) => warn!("Failed to decrypt {}: {}", db_path.display(), e),
            }
        }
        
        if decrypted_paths.is_empty() {
            return Err(WxCoreError::Generic("No databases were successfully decrypted".to_string()));
        }
        
        // Merge the decrypted databases
        let db_paths_str = decrypted_paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(",");
        
        let merged_db = merge_db(&db_paths_str, out_path)?;
        
        // Clean up temporary directory
        fs::remove_dir_all(temp_dir)?;
        
        Ok(merged_db)
    })
}

/// Merge real-time WeChat databases
pub fn merge_real_time_db(
    key: &str,
    db_paths: &[PathBuf],
    out_path: impl AsRef<Path>,
) -> WxCoreResult<PathBuf> {
    // This is similar to decrypt_merge, but for real-time databases
    decrypt_merge(key, db_paths, out_path)
}

/// Merge all real-time WeChat databases
pub fn all_merge_real_time_db(
    key: &str,
    db_paths: &[PathBuf],
    out_path: impl AsRef<Path>,
) -> WxCoreResult<PathBuf> {
    // This is similar to decrypt_merge, but for all real-time databases
    decrypt_merge(key, db_paths, out_path)
}
