pub mod commands;
pub mod gallery;
pub mod logging;
pub mod mcp;
pub mod paths;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let logging_guard = logging::init();
    tracing::info!(target: "sidecar", "app starting");

    tauri::Builder::default()
        .manage(logging_guard)
        .manage(gallery::state::GalleryState::default())
        .manage(mcp::http_server::McpServerState::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::gallery::list_gallery_view,
            commands::gallery::get_mcp_server_status,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = mcp::http_server::start(app_handle).await {
                    tracing::error!(target: "sidecar", ?error, "MCP server stopped with error");
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
