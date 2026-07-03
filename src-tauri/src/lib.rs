pub mod app_paths;
pub mod commands;
pub mod gallery;
pub mod logging;
pub mod mcp;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_paths = app_paths::AppPaths::init().expect("failed to initialize app data paths");
    let logging_guard = logging::init(&app_paths);
    tracing::info!(
        target: "sidecar",
        data_dir = %app_paths.data_dir.display(),
        gallery_state = %app_paths.gallery_state_path.display(),
        gallery_events = %app_paths.gallery_events_path.display(),
        artifacts_dir = %app_paths.artifacts_dir.display(),
        "app starting"
    );

    let gallery_state =
        tauri::async_runtime::block_on(gallery::state::GalleryState::load(app_paths.clone()))
            .expect("failed to load gallery state");

    tauri::Builder::default()
        .manage(logging_guard)
        .manage(app_paths)
        .manage(gallery_state)
        .manage(mcp::http_server::McpServerState::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::gallery::list_gallery_view,
            commands::gallery::set_sidebar_mode,
            commands::gallery::select_session,
            commands::gallery::toggle_turn_collapsed,
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
