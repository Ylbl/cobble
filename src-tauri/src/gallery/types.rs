use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactSession {
    pub id: String,
    pub title: String,
    pub source_kind: SessionSource,
    pub client_name: ClientName,
    pub group_name: String,
    pub project_name: String,
    pub project_path: String,
    pub created_at: String,
    pub updated_at: String,
    pub turns: Vec<ArtifactTurn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SessionSource {
    Mcp,
    Manual,
    Mock,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub enum ClientName {
    Codex,
    ZCode,
    Cursor,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactTurn {
    pub id: String,
    pub index: u32,
    pub hint: Option<String>,
    pub created_at: String,
    pub artifacts: Vec<ArtifactItem>,
    pub collapsed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactItem {
    pub id: String,
    pub title: String,
    pub kind: ArtifactKind,
    pub status: ArtifactStatus,
    pub image_url: Option<String>,
    pub local_file_path: Option<String>,
    pub asset_url: Option<String>,
    pub pdf_url: Option<String>,
    pub pdf_local_file_path: Option<String>,
    pub pdf_asset_url: Option<String>,
    pub log_file_path: Option<String>,
    pub stdout_path: Option<String>,
    pub stderr_path: Option<String>,
    pub svg: Option<String>,
    pub latex_code: Option<String>,
    pub source_file_path: Option<String>,
    pub source_text: Option<String>,
    pub mime_type: Option<String>,
    pub file_extension: Option<String>,
    pub latex_engine: Option<crate::settings::types::LatexEngine>,
    pub compile_elapsed_ms: Option<u128>,
    pub error_message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ArtifactKind {
    Image,
    Pdf,
    Latex,
    Svg,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ArtifactStatus {
    Received,
    Rendering,
    Compiling,
    Finished,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum SidebarMode {
    Groups,
    Projects,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexProjectGroup {
    pub id: String,
    pub name: String,
    pub path: String,
}

pub const DEFAULT_GROUP_NAME: &str = "默认分组";
