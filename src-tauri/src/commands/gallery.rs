use tauri::{Emitter, State};

use crate::{
    app_paths::AppPaths,
    gallery::{state::GalleryState, types::SidebarMode, view_model::GalleryView},
    latex::{compiler, doctor, types::LatexDoctorReport},
    mcp::http_server::{self, McpServerState, McpServerStatus},
    settings::{
        state::ConfigState,
        types::{SidecarConfig, SidecarConfigView},
    },
};

#[tauri::command]
pub async fn list_gallery_view(state: State<'_, GalleryState>) -> Result<GalleryView, String> {
    tracing::debug!(target: "sidecar", "list_gallery_view command");
    Ok(state.list_view().await)
}

#[tauri::command]
pub async fn set_sidebar_mode(
    state: State<'_, GalleryState>,
    mode: SidebarMode,
) -> Result<GalleryView, String> {
    tracing::debug!(target: "sidecar", "set_sidebar_mode command");
    state
        .set_sidebar_mode(mode)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn select_session(
    state: State<'_, GalleryState>,
    session_id: String,
) -> Result<GalleryView, String> {
    tracing::debug!(target: "sidecar", session_id = %session_id, "select_session command");
    state
        .select_session(session_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn toggle_turn_collapsed(
    state: State<'_, GalleryState>,
    session_id: String,
    turn_id: String,
) -> Result<GalleryView, String> {
    tracing::debug!(
        target: "sidecar",
        session_id = %session_id,
        turn_id = %turn_id,
        "toggle_turn_collapsed command"
    );
    state
        .toggle_turn_collapsed(session_id, turn_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_mcp_server_status(
    state: State<'_, McpServerState>,
) -> Result<McpServerStatus, String> {
    tracing::debug!(target: "sidecar", "get_mcp_server_status command");
    Ok(state.status.read().await.clone())
}

#[tauri::command]
pub async fn get_sidecar_config(
    config_state: State<'_, ConfigState>,
) -> Result<SidecarConfigView, String> {
    tracing::debug!(target: "sidecar", "get_sidecar_config command");
    Ok(config_state.get_config_view().await)
}

#[tauri::command]
pub async fn update_sidecar_config(
    app_handle: tauri::AppHandle,
    config_state: State<'_, ConfigState>,
    config: SidecarConfig,
) -> Result<SidecarConfigView, String> {
    tracing::debug!(target: "sidecar", "update_sidecar_config command");
    let view = config_state
        .update_config(config)
        .await
        .map_err(|error| error.to_string())?;
    if let Err(error) = app_handle.emit("config-updated", view.clone()) {
        tracing::error!(target: "sidecar", ?error, "failed to emit config-updated");
    }
    Ok(view)
}

#[tauri::command]
pub async fn restart_mcp_server(
    app_handle: tauri::AppHandle,
    _config_state: State<'_, ConfigState>,
    _mcp_state: State<'_, McpServerState>,
) -> Result<McpServerStatus, String> {
    tracing::debug!(target: "sidecar", "restart_mcp_server command");
    http_server::restart(app_handle)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn open_path(path: String) -> Result<(), String> {
    tracing::debug!(target: "sidecar", path = %path, "open_path command");
    open::that(path).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn run_latex_environment_check(
    config_state: State<'_, ConfigState>,
) -> Result<LatexDoctorReport, String> {
    tracing::debug!(target: "sidecar", "run_latex_environment_check command");
    let config = config_state.get_config().await;
    Ok(doctor::environment_report(&config).await)
}

#[tauri::command]
pub async fn run_latex_smoke_test(
    app_handle: tauri::AppHandle,
    app_paths: State<'_, AppPaths>,
    config_state: State<'_, ConfigState>,
    gallery_state: State<'_, GalleryState>,
) -> Result<GalleryView, String> {
    tracing::debug!(target: "sidecar", "run_latex_smoke_test command");
    let config = config_state.get_config().await;
    let compile_result = compiler::run_smoke_test(
        &app_paths,
        config.latex.engine.clone(),
        config.latex.compile_timeout_seconds,
    )
    .await
    .map_err(|error| error.to_string())?;
    let view = gallery_state
        .record_latex_smoke_test(compile_result)
        .await
        .map_err(|error| error.to_string())?;
    if let Err(error) = app_handle.emit("gallery-updated", view.clone()) {
        tracing::error!(target: "sidecar", ?error, "failed to emit gallery-updated");
    }
    Ok(view)
}
