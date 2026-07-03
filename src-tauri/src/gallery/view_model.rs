use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::gallery::{
    groups::{build_group_views, build_project_views},
    types::{
        ArtifactKind, ArtifactSession, ArtifactStatus, ArtifactTurn, ClientName, CodexProjectGroup,
        SessionSource, SidebarMode,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryView {
    pub sidebar_mode: SidebarModeView,
    pub groups: Vec<GalleryGroupView>,
    pub projects: Vec<GalleryProjectView>,
    pub sessions: Vec<GallerySessionView>,
    pub selected_session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryGroupView {
    pub id: String,
    pub name: String,
    pub session_ids: Vec<String>,
    pub session_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryProjectView {
    pub id: String,
    pub name: String,
    pub path: String,
    pub session_ids: Vec<String>,
    pub session_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GallerySessionView {
    pub id: String,
    pub title: String,
    pub source_kind: SessionSourceView,
    pub client_name: ClientNameView,
    pub group_name: String,
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
    pub local_file_path: Option<String>,
    pub asset_url: Option<String>,
    pub pdf_url: Option<String>,
    pub pdf_local_file_path: Option<String>,
    pub log_file_path: Option<String>,
    pub stdout_path: Option<String>,
    pub stderr_path: Option<String>,
    pub svg: Option<String>,
    pub latex_code: Option<String>,
    pub mime_type: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub enum ArtifactKindView {
    Image,
    Pdf,
    Latex,
    Svg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArtifactStatusView {
    Received,
    Rendering,
    Finished,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArtifactPreviewTypeView {
    Large,
    Small,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SidebarModeView {
    Groups,
    Projects,
}

pub fn to_gallery_view(
    sessions: Vec<ArtifactSession>,
    selected_session_id: Option<String>,
    sidebar_mode: SidebarMode,
    codex_project_groups: Vec<CodexProjectGroup>,
) -> GalleryView {
    let selected_session_id =
        selected_session_id.or_else(|| sessions.first().map(|s| s.id.clone()));
    let groups = build_group_views(&sessions);
    let projects = build_project_views(&sessions, &codex_project_groups);
    tracing::debug!(
        target: "sidecar",
        session_count = sessions.len(),
        group_count = groups.len(),
        project_count = projects.len(),
        "GalleryView generated"
    );
    GalleryView {
        sidebar_mode: sidebar_mode_to_view(sidebar_mode),
        groups,
        projects,
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
        client_name: client_to_view(session.client_name),
        group_name: session.group_name,
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
                local_file_path: artifact.local_file_path,
                asset_url: artifact.asset_url,
                pdf_url: artifact.pdf_url,
                pdf_local_file_path: artifact.pdf_local_file_path,
                log_file_path: artifact.log_file_path,
                stdout_path: artifact.stdout_path,
                stderr_path: artifact.stderr_path,
                svg: artifact.svg,
                latex_code: artifact.latex_code,
                mime_type: artifact.mime_type,
                error_message: artifact.error_message,
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

fn client_to_view(client_name: ClientName) -> ClientNameView {
    match client_name {
        ClientName::ZCode => ClientNameView::ZCode,
        ClientName::Codex => ClientNameView::Codex,
        ClientName::Cursor => ClientNameView::Cursor,
        ClientName::Unknown => ClientNameView::Unknown,
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

fn sidebar_mode_to_view(mode: SidebarMode) -> SidebarModeView {
    match mode {
        SidebarMode::Groups => SidebarModeView::Groups,
        SidebarMode::Projects => SidebarModeView::Projects,
    }
}
