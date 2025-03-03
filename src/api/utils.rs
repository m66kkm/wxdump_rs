use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::process::Command;
use serde::{Serialize, Deserialize};
use log::{info, warn, error};

use crate::wx_core::utils::{WxCoreError, WxCoreResult, wx_core_error};

/// Open a URL in the default browser
pub fn open_browser(url: &str) -> WxCoreResult<()> {
    wx_core_error(|| {
        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(["/c", "start", url])
                .spawn()?;
        }
        
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(url)
                .spawn()?;
        }
        
        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open")
                .arg(url)
                .spawn()?;
        }
        
        Ok(())
    })
}

/// Get the local IP address
pub fn get_local_ip() -> WxCoreResult<String> {
    wx_core_error(|| {
        let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
        socket.connect("8.8.8.8:80")?;
        let local_addr = socket.local_addr()?;
        
        Ok(local_addr.ip().to_string())
    })
}

/// Check if a port is available
pub fn is_port_available(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
}

/// Find an available port
pub fn find_available_port(start_port: u16) -> WxCoreResult<u16> {
    wx_core_error(|| {
        let mut port = start_port;
        
        while !is_port_available(port) {
            port += 1;
            
            if port > 65535 {
                return Err(WxCoreError::Generic("No available ports found".to_string()));
            }
        }
        
        Ok(port)
    })
}

/// Get the MIME type of a file
pub fn get_mime_type(path: impl AsRef<Path>) -> &'static str {
    let path = path.as_ref();
    
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") | Some("htm") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("txt") => "text/plain",
        Some("pdf") => "application/pdf",
        Some("zip") => "application/zip",
        Some("mp3") => "audio/mpeg",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("ogg") => "audio/ogg",
        Some("wav") => "audio/wav",
        Some("webp") => "image/webp",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        Some("eot") => "application/vnd.ms-fontobject",
        _ => "application/octet-stream",
    }
}

/// Format a timestamp as a human-readable date string
pub fn format_timestamp(timestamp: i64) -> String {
    let dt = chrono::DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap())
        .naive_local();
    
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Format a file size as a human-readable string
pub fn format_file_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if size < KB {
        format!("{} B", size)
    } else if size < MB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else if size < GB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else {
        format!("{:.2} GB", size as f64 / GB as f64)
    }
}
