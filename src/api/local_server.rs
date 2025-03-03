use axum::{
    extract::State,
    response::IntoResponse,
    routing::get,
    Json,
    Router,
};
use log::info;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::wx_core::utils::{wx_core_error, WxCoreError, WxCoreResult};

/// Application state
struct AppState {
    merge_path: Option<PathBuf>,
    wx_path: Option<PathBuf>,
    my_wxid: Option<String>,
}

/// Start the web server
pub async fn start_server_async(
    merge_path: Option<PathBuf>,
    wx_path: Option<PathBuf>,
    my_wxid: Option<String>,
    online: bool,
    port: u16,
    debug: bool,
    is_open_browser: bool,
) -> WxCoreResult<()> {
    wx_core_error(|| {
        // Create application state
        let state = Arc::new(Mutex::new(AppState {
            merge_path,
            wx_path,
            my_wxid,
        }));
        
        // Create router
        let app: Router<()> = Router::new()
            .route("/api/health", get(health_check))
            .route("/api/info", get(get_info))
            .with_state(state);
        
        // TODO: Add more routes
        
        // Determine address to bind to
        let addr = if online {
            SocketAddr::from(([0, 0, 0, 0], port))
        } else {
            SocketAddr::from(([127, 0, 0, 1], port))
        };
        
        // Print server information
        info!("Starting server on http://{}", addr);
        
        // Open browser if requested
        if is_open_browser {
            let url = format!("http://localhost:{}", port);
            // TODO: Open browser
        }
        
        Ok(())
    })
}

/// Start the web server (blocking)
pub fn start_server(
    merge_path: Option<PathBuf>,
    wx_path: Option<PathBuf>,
    my_wxid: Option<String>,
    online: bool,
    port: u16,
    debug: bool,
    is_open_browser: bool,
) -> WxCoreResult<()> {
    // Create a runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| WxCoreError::Generic(format!("Failed to create runtime: {}", e)))?;
    
    // Run the async function
    runtime.block_on(async {
        start_server_async(merge_path, wx_path, my_wxid, online, port, debug, is_open_browser).await
    })
}

/// Generate a FastAPI app
pub fn gen_fastapi_app() -> WxCoreResult<()> {
    // This is a placeholder for the Python FastAPI app generation
    // In the Rust version, we're using Axum instead
    Ok(())
}

/// Health check handler
async fn health_check() -> &'static str {
    "OK"
}

/// Get information handler
async fn get_info(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let state = state.lock().unwrap();
    
    let merge_path = state.merge_path.as_ref().map(|p| p.to_string_lossy().to_string());
    let wx_path = state.wx_path.as_ref().map(|p| p.to_string_lossy().to_string());
    let my_wxid = state.my_wxid.clone();
    
    Json(serde_json::json!({
        "merge_path": merge_path,
        "wx_path": wx_path,
        "my_wxid": my_wxid,
    }))
}
