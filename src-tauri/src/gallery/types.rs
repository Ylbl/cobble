use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactSession {
    pub id: String,
    pub title: String,
    pub source_kind: SessionSource,
    pub client_name: String,
    pub project_name: String,
    pub project_path: String,
    pub created_at: String,
    pub updated_at: String,
    pub turns: Vec<ArtifactTurn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionSource {
    Mcp,
    Manual,
    Mock,
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
    pub pdf_url: Option<String>,
    pub svg: Option<String>,
    pub latex_code: Option<String>,
    pub source_text: Option<String>,
    pub mime_type: Option<String>,
    pub file_extension: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactKind {
    Image,
    Pdf,
    Latex,
    Svg,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactStatus {
    Received,
    Rendering,
    Finished,
    Failed,
}
