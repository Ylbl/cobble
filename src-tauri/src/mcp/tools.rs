use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, Json, ServerHandler,
};
use tauri::Manager;

use crate::{
    gallery::{debug_artifacts, events, state::GalleryState},
    mcp::types::{DisplayArtifactTurnInput, DisplayArtifactTurnResult},
};

const MCP_INSTRUCTIONS: &str = r#"This server displays artifacts in a running Sidecar App.

Use display_artifact_turn when the user asks to display an image, PDF, LaTeX artifact, diagram, circuit, chemical structure, or visual output.

One assistant response should call display_artifact_turn once.

Put all artifacts from the same assistant response into the artifacts array.

If a previous tool result in this conversation contains sidecarSessionId, reuse that exact sidecarSessionId in later display_artifact_turn calls.

If no sidecarSessionId exists, omit sidecarSessionId and provide a clear sessionTitle.

Do not invent sidecarSessionId.

First version supports image artifacts through imageUrl.

LaTeX and PDF support will be implemented later."#;

#[derive(Debug, Clone)]
pub struct SidecarMcpService {
    app: tauri::AppHandle,
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

impl SidecarMcpService {
    pub fn new(app: tauri::AppHandle) -> Self {
        Self {
            app,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl SidecarMcpService {
    #[tool(
        name = "display_artifact_turn",
        description = "Display image artifacts in the running Sidecar App. One call represents one assistant response turn."
    )]
    pub async fn display_artifact_turn(
        &self,
        Parameters(input): Parameters<DisplayArtifactTurnInput>,
    ) -> Result<Json<DisplayArtifactTurnResult>, rmcp::ErrorData> {
        tracing::info!(
            target: "sidecar",
            artifact_count = input.artifacts.len(),
            has_sidecar_session_id = input.sidecar_session_id.is_some(),
            "MCP tool display_artifact_turn called"
        );

        let state = self.app.state::<GalleryState>();
        let snapshot_before = state.snapshot().await;
        let result = state.display_artifact_turn(input.clone()).await;
        let snapshot_after = state.snapshot().await;
        let view = state.list_view().await;
        if let Err(error) = debug_artifacts::write_run(
            state.app_paths(),
            &input,
            &result,
            &snapshot_before,
            &snapshot_after,
            &view,
        ) {
            tracing::error!(target: "sidecar", ?error, "failed to write debug artifacts");
        }
        events::emit_artifact_created(&self.app, &result);
        events::emit_gallery_updated(&self.app, view);

        Ok(Json(result))
    }
}

#[tool_handler]
impl ServerHandler for SidecarMcpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(MCP_INSTRUCTIONS)
    }
}
