use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::gallery::types::{
    ArtifactKind, ArtifactSession, ArtifactStatus, ArtifactTurn, SessionSource,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryView {
    pub sessions: Vec<GallerySessionView>,
    pub selected_session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GallerySessionView {
    pub id: String,
    pub title: String,
    pub source_kind: SessionSourceView,
    pub client_name: ClientNameView,
    pub project_name: String,
    pub project_path: String,
    pub created_at: String,
    pub updated_at: String,
    pub updated_at_label: String,
    pub artifact_count: usize,
    pub turns: Vec<GalleryTurnView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryTurnView {
    pub id: String,
    pub index: u32,
    pub hint: String,
    pub created_at: String,
    pub collapsed: bool,
    pub artifacts: Vec<GalleryArtifactView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryArtifactView {
    pub id: String,
    pub title: String,
    pub kind: ArtifactKindView,
    pub status: ArtifactStatusView,
    pub preview_type: ArtifactPreviewTypeView,
    pub image_url: Option<String>,
    pub pdf_url: Option<String>,
    pub svg: Option<String>,
    pub latex_code: Option<String>,
    pub mime_type: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionSourceView {
    Mcp,
    Manual,
    Mock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientNameView {
    ZCode,
    Codex,
    Cursor,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactKindView {
    Image,
    Pdf,
    Latex,
    Svg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactStatusView {
    Received,
    Rendering,
    Finished,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactPreviewTypeView {
    Large,
    Small,
}

pub fn to_gallery_view(
    sessions: Vec<ArtifactSession>,
    selected_session_id: Option<String>,
) -> GalleryView {
    let selected_session_id =
        selected_session_id.or_else(|| sessions.first().map(|s| s.id.clone()));
    GalleryView {
        sessions: sessions.into_iter().map(to_session_view).collect(),
        selected_session_id,
    }
}

fn to_session_view(session: ArtifactSession) -> GallerySessionView {
    let artifact_count = session.turns.iter().map(|turn| turn.artifacts.len()).sum();

    GallerySessionView {
        id: session.id,
        title: session.title,
        source_kind: source_to_view(session.source_kind),
        client_name: client_to_view(&session.client_name),
        project_name: session.project_name,
        project_path: session.project_path,
        created_at: session.created_at,
        updated_at_label: format_time_label(&session.updated_at),
        updated_at: session.updated_at,
        artifact_count,
        turns: session.turns.into_iter().map(to_turn_view).collect(),
    }
}

fn format_time_label(value: &str) -> String {
    DateTime::parse_from_rfc3339(value)
        .map(|time| time.with_timezone(&Local).format("%H:%M").to_string())
        .unwrap_or_default()
}

fn to_turn_view(turn: ArtifactTurn) -> GalleryTurnView {
    GalleryTurnView {
        id: turn.id,
        index: turn.index,
        hint: turn.hint.unwrap_or_default(),
        created_at: turn.created_at,
        collapsed: turn.collapsed,
        artifacts: turn
            .artifacts
            .into_iter()
            .enumerate()
            .map(|(index, artifact)| GalleryArtifactView {
                id: artifact.id,
                title: artifact.title,
                kind: kind_to_view(artifact.kind),
                status: status_to_view(artifact.status),
                preview_type: if index == 0 {
                    ArtifactPreviewTypeView::Large
                } else {
                    ArtifactPreviewTypeView::Small
                },
                image_url: artifact.image_url,
                pdf_url: artifact.pdf_url,
                svg: artifact.svg,
                latex_code: artifact.latex_code,
                mime_type: artifact.mime_type,
                created_at: artifact.created_at,
            })
            .collect(),
    }
}

fn source_to_view(source: SessionSource) -> SessionSourceView {
    match source {
        SessionSource::Mcp => SessionSourceView::Mcp,
        SessionSource::Manual => SessionSourceView::Manual,
        SessionSource::Mock => SessionSourceView::Mock,
    }
}

fn client_to_view(client_name: &str) -> ClientNameView {
    match client_name {
        "ZCode" => ClientNameView::ZCode,
        "Codex" => ClientNameView::Codex,
        "Cursor" => ClientNameView::Cursor,
        _ => ClientNameView::Unknown,
    }
}

fn kind_to_view(kind: ArtifactKind) -> ArtifactKindView {
    match kind {
        ArtifactKind::Image => ArtifactKindView::Image,
        ArtifactKind::Pdf => ArtifactKindView::Pdf,
        ArtifactKind::Latex => ArtifactKindView::Latex,
        ArtifactKind::Svg => ArtifactKindView::Svg,
    }
}

fn status_to_view(status: ArtifactStatus) -> ArtifactStatusView {
    match status {
        ArtifactStatus::Received => ArtifactStatusView::Received,
        ArtifactStatus::Rendering => ArtifactStatusView::Rendering,
        ArtifactStatus::Finished => ArtifactStatusView::Finished,
        ArtifactStatus::Failed => ArtifactStatusView::Failed,
    }
}
