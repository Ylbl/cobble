use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    gallery::types::{
        ArtifactItem, ArtifactKind, ArtifactSession, ArtifactStatus, ArtifactTurn, SessionSource,
    },
    gallery::view_model::{to_gallery_view, GalleryView},
    mcp::types::{
        ArtifactInputKind, DisplayArtifactTurnInput, DisplayArtifactTurnResult, REUSE_INSTRUCTION,
    },
};

pub struct GalleryState {
    pub sessions: RwLock<Vec<ArtifactSession>>,
    pub selected_session_id: RwLock<Option<String>>,
}

impl Default for GalleryState {
    fn default() -> Self {
        Self {
            sessions: RwLock::new(Vec::new()),
            selected_session_id: RwLock::new(None),
        }
    }
}

impl GalleryState {
    pub async fn list_sessions(&self) -> Vec<ArtifactSession> {
        tracing::debug!(target: "sidecar", "front end requested gallery state");
        self.sessions.read().await.clone()
    }

    pub async fn list_view(&self) -> GalleryView {
        tracing::debug!(target: "sidecar", "front end requested gallery view");
        let sessions = self.sessions.read().await.clone();
        let selected_session_id = self.selected_session_id.read().await.clone();
        to_gallery_view(sessions, selected_session_id)
    }

    pub async fn display_artifact_turn(
        &self,
        input: DisplayArtifactTurnInput,
    ) -> DisplayArtifactTurnResult {
        tracing::info!(target: "sidecar", artifact_count = input.artifacts.len(), "display_artifact_turn received");

        let mut sessions = self.sessions.write().await;
        let mut created_new_session = false;
        let session_index = input
            .sidecar_session_id
            .as_ref()
            .and_then(|id| sessions.iter().position(|session| &session.id == id));

        let session_index = match session_index {
            Some(index) => {
                tracing::debug!(target: "sidecar", sidecar_session_id = %sessions[index].id, "reusing session");
                index
            }
            None => {
                created_new_session = true;
                let now = now_string();
                let id = Uuid::new_v4().to_string();
                tracing::info!(target: "sidecar", sidecar_session_id = %id, "creating session");
                sessions.push(ArtifactSession {
                    id,
                    title: input
                        .session_title
                        .clone()
                        .unwrap_or_else(|| "未命名会话".to_string()),
                    source_kind: SessionSource::Mcp,
                    client_name: "Unknown".to_string(),
                    project_name: String::new(),
                    project_path: String::new(),
                    created_at: now.clone(),
                    updated_at: now,
                    turns: Vec::new(),
                });
                sessions.len() - 1
            }
        };

        let session = &mut sessions[session_index];
        if let Some(session_title) = input
            .session_title
            .as_ref()
            .map(|title| title.trim())
            .filter(|title| !title.is_empty())
        {
            if session.title != session_title {
                tracing::info!(
                    target: "sidecar",
                    sidecar_session_id = %session.id,
                    old_title = %session.title,
                    new_title = %session_title,
                    "updating session title"
                );
                session.title = session_title.to_string();
            }
        }

        for turn in &mut session.turns {
            turn.collapsed = true;
        }

        let now = now_string();
        let turn_id = Uuid::new_v4().to_string();
        let turn_index = session.turns.len() as u32 + 1;
        let mut artifact_ids = Vec::new();
        let mut artifacts = Vec::new();

        tracing::info!(
            target: "sidecar",
            sidecar_session_id = %session.id,
            sidecar_turn_id = %turn_id,
            turn_index,
            "creating turn"
        );

        for artifact in input.artifacts {
            match artifact.kind {
                ArtifactInputKind::Image => {
                    let artifact_id = Uuid::new_v4().to_string();
                    let status = if artifact.image_url.is_some() {
                        ArtifactStatus::Finished
                    } else {
                        ArtifactStatus::Failed
                    };
                    tracing::debug!(
                        target: "sidecar",
                        artifact_id = %artifact_id,
                        title = %artifact.title,
                        "creating image artifact"
                    );
                    artifact_ids.push(artifact_id.clone());
                    artifacts.push(ArtifactItem {
                        id: artifact_id,
                        title: artifact.title,
                        kind: ArtifactKind::Image,
                        status,
                        image_url: artifact.image_url,
                        pdf_url: None,
                        svg: None,
                        latex_code: None,
                        source_text: None,
                        mime_type: Some("image/png".to_string()),
                        file_extension: Some("png".to_string()),
                        created_at: now.clone(),
                    });
                }
                unsupported => {
                    tracing::warn!(target: "sidecar", ?unsupported, "unsupported artifact kind ignored");
                }
            }
        }

        session.updated_at = now.clone();
        session.turns.push(ArtifactTurn {
            id: turn_id.clone(),
            index: turn_index,
            hint: input.turn_hint,
            created_at: now,
            artifacts,
            collapsed: false,
        });
        let sidecar_session_id = session.id.clone();
        *self.selected_session_id.write().await = Some(sidecar_session_id.clone());

        let displayed = !artifact_ids.is_empty();
        tracing::info!(
            target: "sidecar",
            sidecar_session_id = %session.id,
            sidecar_turn_id = %turn_id,
            artifact_count = artifact_ids.len(),
            "display_artifact_turn completed"
        );

        DisplayArtifactTurnResult {
            ok: true,
            sidecar_session_id,
            sidecar_turn_id: turn_id,
            artifact_ids,
            created_new_session,
            displayed,
            message: if displayed {
                "Artifacts displayed in Sidecar.".to_string()
            } else {
                "No supported image artifacts were provided.".to_string()
            },
            reuse_instruction: REUSE_INSTRUCTION.to_string(),
        }
    }
}

fn now_string() -> String {
    Utc::now().to_rfc3339()
}
