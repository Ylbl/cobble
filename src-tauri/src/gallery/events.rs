use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::{gallery::view_model::GalleryView, mcp::types::DisplayArtifactTurnResult};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtifactCreatedPayload {
    sidecar_session_id: String,
    sidecar_turn_id: String,
    artifact_ids: Vec<String>,
}

pub fn emit_gallery_updated(app: &AppHandle, view: GalleryView) {
    tracing::debug!(target: "sidecar", session_count = view.sessions.len(), "emitting gallery-updated");
    if let Err(error) = app.emit("gallery-updated", view) {
        tracing::error!(target: "sidecar", ?error, "failed to emit gallery-updated");
    }
}

pub fn emit_artifact_created(app: &AppHandle, result: &DisplayArtifactTurnResult) {
    tracing::debug!(
        target: "sidecar",
        sidecar_session_id = %result.sidecar_session_id,
        sidecar_turn_id = %result.sidecar_turn_id,
        "emitting artifact-created"
    );
    if let Err(error) = app.emit(
        "artifact-created",
        ArtifactCreatedPayload {
            sidecar_session_id: result.sidecar_session_id.clone(),
            sidecar_turn_id: result.sidecar_turn_id.clone(),
            artifact_ids: result.artifact_ids.clone(),
        },
    ) {
        tracing::error!(target: "sidecar", ?error, "failed to emit artifact-created");
    }
}
