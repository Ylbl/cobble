use tauri::State;

use crate::{
    gallery::{state::GalleryState, view_model::GalleryView},
    mcp::http_server::{McpServerState, McpServerStatus},
};

#[tauri::command]
pub async fn list_gallery_view(state: State<'_, GalleryState>) -> Result<GalleryView, String> {
    tracing::debug!(target: "sidecar", "list_gallery_view command");
    Ok(state.list_view().await)
}

#[tauri::command]
pub async fn get_mcp_server_status(
    state: State<'_, McpServerState>,
) -> Result<McpServerStatus, String> {
    tracing::debug!(target: "sidecar", "get_mcp_server_status command");
    Ok(state.status.read().await.clone())
}
