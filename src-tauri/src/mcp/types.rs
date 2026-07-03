use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DisplayArtifactTurnInput {
    pub sidecar_session_id: Option<String>,
    pub session_title: Option<String>,
    pub turn_hint: Option<String>,
    pub artifacts: Vec<ArtifactInput>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
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
    pub created_new_session: bool,
    pub displayed: bool,
    pub message: String,
    pub reuse_instruction: String,
}

pub const REUSE_INSTRUCTION: &str =
    "Reuse this sidecarSessionId in later display_artifact_turn calls for the same AI conversation.";
