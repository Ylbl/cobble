use std::{io::Write, path::Path};

use anyhow::{Context, Result};
use atomic_write_file::AtomicWriteFile;

use crate::settings::types::SidecarConfig;

pub fn load_or_create_config(path: &Path) -> Result<SidecarConfig> {
    tracing::info!(target: "sidecar", path = %path.display(), "loading sidecar.config.json");

    if !path.exists() {
        tracing::info!(target: "sidecar", path = %path.display(), "sidecar.config.json does not exist; creating default config");
        let config = SidecarConfig::default();
        save_config_atomic_sync(path, &config)?;
        return Ok(config);
    }

    let content =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;

    let raw: serde_json::Value = match serde_json::from_str(content.trim_start_matches('\u{feff}')) {
        Ok(value) => value,
        Err(error) => {
            tracing::error!(target: "sidecar", path = %path.display(), ?error, "failed to parse sidecar.config.json; backing up and recreating");
            backup_and_recreate_default(path, &content)?;
            return Ok(SidecarConfig::default());
        }
    };

    let mut config = SidecarConfig::default();

    if let Some(mcp) = raw.get("mcp") {
        match serde_json::from_value(mcp.clone()) {
            Ok(m) => config.mcp = m,
            Err(error) => {
                tracing::warn!(target: "sidecar", ?error, "failed to parse mcp section; using default");
            }
        }
    }

    if let Some(latex) = raw.get("latex") {
        match serde_json::from_value(latex.clone()) {
            Ok(l) => config.latex = l,
            Err(error) => {
                tracing::warn!(target: "sidecar", ?error, "failed to parse latex section; using default");
            }
        }
    }

    if let Some(gallery) = raw.get("gallery") {
        match serde_json::from_value(gallery.clone()) {
            Ok(g) => config.gallery = g,
            Err(error) => {
                tracing::warn!(target: "sidecar", ?error, "failed to parse gallery section; using default");
            }
        }
    }

    // Detect legacy fields for informational logging
    for legacy_key in &["instanceName", "paths"] {
        if raw.get(legacy_key).is_some() {
            tracing::info!(target: "sidecar", key = legacy_key, "legacy config field ignored (will be removed on next save)");
        }
    }

    tracing::info!(target: "sidecar", path = %path.display(), "sidecar.config.json loaded");
    Ok(config)
}

fn backup_and_recreate_default(path: &Path, content: &str) -> Result<()> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let backup_path = path.with_file_name(format!(
        "sidecar.config.invalid.{}.json",
        timestamp
    ));

    // If the existing file is empty, don't bother backing up
    if !content.trim().is_empty() {
        std::fs::rename(path, &backup_path)
            .with_context(|| format!("backing up invalid config to {}", backup_path.display()))?;
        tracing::warn!(target: "sidecar", original = %path.display(), backup = %backup_path.display(), "invalid config backed up");
    }

    let config = SidecarConfig::default();
    save_config_atomic_sync(path, &config)?;
    tracing::info!(target: "sidecar", path = %path.display(), "recreated default sidecar.config.json");
    Ok(())
}

pub async fn save_config_atomic(path: &Path, config: &SidecarConfig) -> Result<()> {
    let path = path.to_path_buf();
    let config = config.clone();
    tokio::task::spawn_blocking(move || save_config_atomic_sync(&path, &config)).await??;
    Ok(())
}

fn save_config_atomic_sync(path: &Path, config: &SidecarConfig) -> Result<()> {
    let json = serde_json::to_vec_pretty(config)?;
    let mut file = AtomicWriteFile::options()
        .open(path)
        .with_context(|| format!("opening atomic writer for {}", path.display()))?;
    file.write_all(&json)
        .with_context(|| format!("writing {}", path.display()))?;
    file.commit()
        .with_context(|| format!("committing {}", path.display()))?;
    tracing::info!(target: "sidecar", path = %path.display(), "sidecar.config.json saved atomically");
    Ok(())
}
