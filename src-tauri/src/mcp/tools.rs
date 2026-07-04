use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, Json, ServerHandler,
};
use tauri::Manager;

use crate::{
    gallery::{debug_artifacts, events, state::GalleryState},
    mcp::types::{DisplayArtifactTurnInput, DisplayArtifactTurnResult},
    settings::state::ConfigState,
};

#[derive(Debug, Clone)]
pub struct SidecarMcpService {
    app: tauri::AppHandle,
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
    instructions: String,
}

impl SidecarMcpService {
    pub fn new(app: tauri::AppHandle, instructions: String) -> Self {
        Self {
            app,
            tool_router: Self::tool_router(),
            instructions,
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
        let config = self.app.state::<ConfigState>().get_config().await;
        let snapshot_before = state.snapshot().await;
        let result = state.display_artifact_turn(input.clone(), config).await;
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
            .with_instructions(self.instructions.clone())
    }
}
