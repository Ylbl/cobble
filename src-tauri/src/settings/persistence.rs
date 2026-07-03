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
    let config = serde_json::from_str(content.trim_start_matches('\u{feff}'))
        .with_context(|| format!("parsing {}", path.display()))?;
    tracing::info!(target: "sidecar", path = %path.display(), "sidecar.config.json loaded");
    Ok(config)
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
