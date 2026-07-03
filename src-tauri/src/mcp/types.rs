use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::settings::types::LatexEngine;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DisplayArtifactTurnInput {
    pub sidecar_session_id: Option<String>,
    pub session_title: Option<String>,
    pub group_name: Option<String>,
    pub client_name: Option<ClientName>,
    pub project_name: Option<String>,
    pub project_path: Option<String>,
    pub turn_hint: Option<String>,
    pub artifacts: Vec<ArtifactInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ClientName {
    Codex,
    ZCode,
    Cursor,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactInput {
    pub title: String,
    pub kind: ArtifactInputKind,
    pub image_url: Option<String>,
    pub latex_code: Option<String>,
    pub pdf_url: Option<String>,
    pub svg: Option<String>,
    pub latex_engine: Option<LatexEngine>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ArtifactInputKind {
    Image,
    Latex,
    Pdf,
    Svg,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DisplayArtifactTurnResult {
    pub ok: bool,
    pub sidecar_session_id: String,
    pub sidecar_turn_id: String,
    pub artifact_ids: Vec<String>,
    pub artifact_results: Vec<ArtifactDisplayResult>,
    pub created_new_session: bool,
    pub displayed: bool,
    pub message: String,
    pub reuse_instruction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactDisplayResult {
    pub artifact_id: String,
    pub kind: String,
    pub status: String,
    pub pdf_generated: bool,
    pub message: String,
}

pub const REUSE_INSTRUCTION: &str =
    "Reuse this sidecarSessionId in later display_artifact_turn calls for the same AI conversation.";
