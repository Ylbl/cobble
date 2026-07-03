use anyhow::Result;

use crate::{app_paths::AppPaths, gallery::types::CodexProjectGroup};

pub async fn load_project_groups(_paths: &AppPaths) -> Result<Vec<CodexProjectGroup>> {
    tracing::info!(target: "sidecar", "loading Codex project groups");
    tracing::info!(target: "sidecar", count = 0, "Codex project groups loaded");
    Ok(Vec::new())
}
