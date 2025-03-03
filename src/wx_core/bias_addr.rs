use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{self, Read, Write};
use serde::{Serialize, Deserialize};
use log::{info, warn, error};

use crate::wx_core::utils::{WxCoreError, WxCoreResult, wx_core_error, WxOffs};
use crate::wx_core::wx_info::{get_wx_info, WxInfo};

#[derive(Debug, Serialize, Deserialize)]
pub struct BiasAddrResult {
    pub version: String,
    pub name_bias: usize,
    pub account_bias: usize,
    pub mobile_bias: usize,
    pub email_bias: usize,
    pub key_bias: usize,
}

pub struct BiasAddr {
    account: String,
    mobile: String,
    name: String,
    key: Option<String>,
    db_path: Option<PathBuf>,
}

impl BiasAddr {
    pub fn new(
        account: String,
        mobile: String,
        name: String,
        key: Option<String>,
        db_path: Option<PathBuf>,
    ) -> Self {
        Self {
            account,
            mobile,
            name,
            key,
            db_path,
        }
    }
    
    pub fn run(&self, is_print: bool, wx_offs_path: Option<PathBuf>) -> WxCoreResult<BiasAddrResult> {
        wx_core_error(|| {
            // Get WeChat information
            let wx_infos = get_wx_info(&None, false, None)?;
            
            if wx_infos.is_empty() {
                return Err(WxCoreError::WeChatNotRunning);
            }
            
            // TODO: Implement the actual logic to get the base address offset
            // This would involve searching memory for the account, mobile, name, and key
            
            // For now, we'll just return a dummy result
            let result = BiasAddrResult {
                version: wx_infos[0].version.clone(),
                name_bias: 0,
                account_bias: 0,
                mobile_bias: 0,
                email_bias: 0,
                key_bias: 0,
            };
            
            // Update WX_OFFS file if provided
            if let Some(path) = wx_offs_path {
                let mut wx_offs = if path.exists() {
                    WxOffs::from_file(&path)?
                } else {
                    WxOffs::new()
                };
                
                wx_offs.add_offsets(
                    result.version.clone(),
                    vec![
                        result.name_bias,
                        result.account_bias,
                        result.mobile_bias,
                        result.email_bias,
                        result.key_bias,
                    ],
                );
                
                wx_offs.to_file(path)?;
            }
            
            // Print results if requested
            if is_print {
                println!("{}", "=".repeat(32));
                println!("[+] {:>8}: {}", "version", result.version);
                println!("[+] {:>8}: {:#x}", "name", result.name_bias);
                println!("[+] {:>8}: {:#x}", "account", result.account_bias);
                println!("[+] {:>8}: {:#x}", "mobile", result.mobile_bias);
                println!("[+] {:>8}: {:#x}", "email", result.email_bias);
                println!("[+] {:>8}: {:#x}", "key", result.key_bias);
                println!("{}", "=".repeat(32));
            }
            
            Ok(result)
        })
    }
}

pub fn run_bias_addr(
    account: String,
    mobile: String,
    name: String,
    key: Option<String>,
    db_path: Option<PathBuf>,
    wx_offs_path: Option<PathBuf>,
) -> WxCoreResult<BiasAddrResult> {
    let bias_addr = BiasAddr::new(account, mobile, name, key, db_path);
    bias_addr.run(true, wx_offs_path)
}
