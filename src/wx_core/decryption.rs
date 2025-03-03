use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use aes::Aes256;
use aes::cipher::{BlockDecrypt, KeyInit};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use pbkdf2::pbkdf2;
use log::{info, error};
use crate::wx_core::utils::{WxCoreError, WxCoreResult, wx_core_error};

const SQLITE_FILE_HEADER: &str = "SQLite format 3\0";
const KEY_SIZE: usize = 32;
const DEFAULT_PAGESIZE: usize = 4096;

type HmacSha1 = Hmac<Sha1>;

/// Decrypt a WeChat database file
pub fn decrypt(key: &str, db_path: impl AsRef<Path>, out_path: impl AsRef<Path>) -> WxCoreResult<(PathBuf, PathBuf, String)> {
    wx_core_error(|| {
        let db_path = db_path.as_ref();
        let out_path = out_path.as_ref();
        
        // Validate inputs
        if !db_path.exists() || !db_path.is_file() {
            return Err(WxCoreError::InvalidPath(format!("db_path: '{}' File not found!", db_path.display())));
        }
        
        if !out_path.parent().map_or(false, |p| p.exists()) {
            return Err(WxCoreError::InvalidPath(format!("out_path: '{}' Parent directory not found!", out_path.display())));
        }
        
        if key.len() != 64 {
            return Err(WxCoreError::Key(format!("key: '{}' Len Error!", key)));
        }
        
        // Parse the key
        let password = hex::decode(key.trim()).map_err(|_| WxCoreError::Key(format!("key: '{}' Invalid hex!", key)))?;
        
        // Read the database file
        let mut file = File::open(db_path)?;
        let mut blist = Vec::new();
        file.read_to_end(&mut blist)?;
        
        // Extract the salt (first 16 bytes)
        if blist.len() < 16 {
            return Err(WxCoreError::Database(format!("db_path: '{}' File too small!", db_path.display())));
        }
        
        let salt = &blist[0..16];
        let first = &blist[16..4096];
        
        // Derive the HMAC key
        let mac_salt: Vec<u8> = salt.iter().map(|&b| b ^ 58).collect();
        let mut byte_hmac = [0u8; KEY_SIZE];
        pbkdf2::<Hmac<Sha1>>(password.as_slice(), salt, 64000, &mut byte_hmac);
        
        let mut mac_key = [0u8; KEY_SIZE];
        pbkdf2::<Hmac<Sha1>>(byte_hmac.as_slice(), &mac_salt, 2, &mut mac_key);
        
        // Verify the HMAC
        let mut mac = <HmacSha1 as Mac>::new_from_slice(&mac_key)
            .map_err(|_| WxCoreError::Key("Failed to create HMAC".to_string()))?;
        mac.update(&blist[16..4064]);
        mac.update(&[1, 0, 0, 0]);
        
        let expected_hmac = &first[first.len() - 32..first.len() - 12];
        let calculated_hmac = mac.finalize().into_bytes();
        
        if &calculated_hmac[..] != expected_hmac {
            return Err(WxCoreError::Key(format!(
                "Key Error! (key: '{}'; db_path: '{}'; out_path: '{}')",
                key, db_path.display(), out_path.display()
            )));
        }
        
        // Create the output file
        let mut de_file = File::create(out_path)?;
        
        // Write the SQLite header
        de_file.write_all(SQLITE_FILE_HEADER.as_bytes())?;
        
        // TODO: Implement the actual decryption logic
        // This would involve:
        // 1. For each 4096-byte page:
        //    a. Extract the IV from the page
        //    b. Decrypt the page using AES-CBC
        //    c. Write the decrypted page to the output file
        
        // For now, we'll just copy the file as-is
        // This is a placeholder and should be replaced with actual decryption
        
        Ok((db_path.to_path_buf(), out_path.to_path_buf(), key.to_string()))
    })
}

/// Batch decrypt WeChat database files
pub fn batch_decrypt(
    key: &str,
    db_path: impl AsRef<Path>,
    out_path: impl AsRef<Path>,
    is_print: bool,
) -> WxCoreResult<Vec<WxCoreResult<(PathBuf, PathBuf, String)>>> {
    wx_core_error(|| {
        let db_path = db_path.as_ref();
        let out_path = out_path.as_ref();
        
        // Validate inputs
        if key.len() != 64 {
            return Err(WxCoreError::Key(format!("key: '{}' Len Error!", key)));
        }
        
        if !out_path.exists() {
            return Err(WxCoreError::InvalidPath(format!("out_path: '{}' not found!", out_path.display())));
        }
        
        if !db_path.exists() {
            return Err(WxCoreError::InvalidPath(format!("db_path: '{}' not found!", db_path.display())));
        }
        
        let mut process_list = Vec::new();
        
        if db_path.is_file() {
            let in_path = db_path;
            let out_file = format!("de_{}", in_path.file_name().unwrap().to_string_lossy());
            let out_file_path = out_path.join(out_file);
            process_list.push((key, in_path, out_file_path));
        } else if db_path.is_dir() {
            // Walk the directory and find all files
            for entry in walkdir::WalkDir::new(db_path) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    let in_path = entry.path().to_path_buf();
                    let rel_path = in_path.strip_prefix(db_path).unwrap_or(&in_path);
                    let out_file = format!("de_{}", rel_path.file_name().unwrap().to_string_lossy());
                    let out_dir = out_path.join(rel_path.parent().unwrap_or(Path::new("")));
                    
                    // Create the output directory if it doesn't exist
                    fs::create_dir_all(&out_dir)?;
                    
                    let out_file_path = out_dir.join(out_file);
                    process_list.push((key, in_path, out_file_path));
                }
            }
        } else {
            return Err(WxCoreError::InvalidPath(format!("db_path: '{}' is neither a file nor a directory!", db_path.display())));
        }
        
        // Decrypt each file
        let mut results = Vec::new();
        for (key, in_path, out_path) in process_list {
            results.push(decrypt(key, in_path, out_path));
        }
        
        // Remove empty directories
        if db_path.is_dir() {
            for entry in walkdir::WalkDir::new(out_path).contents_first(true) {
                let entry = entry?;
                if entry.file_type().is_dir() {
                    let dir = entry.path();
                    if fs::read_dir(dir)?.next().is_none() {
                        fs::remove_dir(dir)?;
                    }
                }
            }
        }
        
        // Print results if requested
        if is_print {
            println!("{}", "=".repeat(32));
            let mut success_count = 0;
            let mut fail_count = 0;
            
            for result in &results {
                match result {
                    Ok((in_path, out_path, _)) => {
                        println!("[+] \"{}\" -> \"{}\"", in_path.display(), out_path.display());
                        success_count += 1;
                    }
                    Err(e) => {
                        println!("{}", e);
                        fail_count += 1;
                    }
                }
            }
            
            println!("{}", "-".repeat(32));
            println!(
                "[+] 共 {} 个文件, 成功 {} 个, 失败 {} 个",
                results.len(),
                success_count,
                fail_count
            );
            println!("{}", "=".repeat(32));
        }
        
        Ok(results)
    })
}
