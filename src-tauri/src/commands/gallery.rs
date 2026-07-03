use tauri::State;

use crate::{
    gallery::{state::GalleryState, types::SidebarMode, view_model::GalleryView},
    mcp::http_server::{McpServerState, McpServerStatus},
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
