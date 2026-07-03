pub mod app_paths;
pub mod commands;
pub mod gallery;
pub mod latex;
pub mod logging;
pub mod mcp;
pub mod settings;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (app_paths, sidecar_config, instance_lock) = match app_paths::AppPaths::bootstrap() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("Sidecar 启动失败：{error:#}");
            std::process::exit(1);
        }
    };
    let logging_guard = logging::init(&app_paths);
    tracing::info!(
        target: "sidecar",
        instance_name = %sidecar_config.instance_name,
        instance_dir = %app_paths.instance_dir.display(),
        config_path = %app_paths.config_path.display(),
        data_dir = %app_paths.data_dir.display(),
        gallery_state = %app_paths.gallery_state_path.display(),
        gallery_events = %app_paths.gallery_events_path.display(),
        artifacts_dir = %app_paths.artifacts_dir.display(),
        "app starting"
    );

    let config_state = settings::state::ConfigState::new(app_paths.clone(), sidecar_config);
    let gallery_state =
        tauri::async_runtime::block_on(gallery::state::GalleryState::load(app_paths.clone()))
            .expect("failed to load gallery state");

    tauri::Builder::default()
        .manage(logging_guard)
        .manage(instance_lock)
        .manage(app_paths)
        .manage(config_state)
        .manage(gallery_state)
        .manage(mcp::http_server::McpServerState::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::gallery::list_gallery_view,
            commands::gallery::set_sidebar_mode,
            commands::gallery::select_session,
            commands::gallery::toggle_turn_collapsed,
            commands::gallery::get_mcp_server_status,
            commands::gallery::get_sidecar_config,
            commands::gallery::update_sidecar_config,
            commands::gallery::restart_mcp_server,
            commands::gallery::open_path,
            commands::gallery::run_latex_environment_check,
            commands::gallery::run_latex_smoke_test,
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
