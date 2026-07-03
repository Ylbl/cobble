use std::io::Write;

use anyhow::{Context, Result};
use atomic_write_file::AtomicWriteFile;
use serde::{Deserialize, Serialize};

use crate::{
    app_paths::AppPaths,
    gallery::types::{ArtifactSession, SidebarMode},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GallerySnapshot {
    pub sessions: Vec<ArtifactSession>,
    pub selected_session_id: Option<String>,
    pub sidebar_mode: SidebarMode,
}

impl Default for GallerySnapshot {
    fn default() -> Self {
        Self {
            sessions: Vec::new(),
            selected_session_id: None,
            sidebar_mode: SidebarMode::Groups,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadedGallerySnapshot {
    pub snapshot: GallerySnapshot,
    pub existed: bool,
}

pub async fn load(paths: &AppPaths) -> Result<LoadedGallerySnapshot> {
    let path = paths.gallery_state_path.clone();
    tracing::info!(target: "sidecar", path = %path.display(), "loading gallery-state.json");

    if !path.exists() {
        tracing::info!(target: "sidecar", path = %path.display(), "gallery-state.json does not exist");
        return Ok(LoadedGallerySnapshot {
            snapshot: GallerySnapshot::default(),
            existed: false,
        });
    }

    let content = tokio::fs::read_to_string(&path)
        .await
        .with_context(|| format!("reading {}", path.display()))?;
    let snapshot =
        serde_json::from_str(&content).with_context(|| format!("parsing {}", path.display()))?;
    tracing::info!(target: "sidecar", path = %path.display(), "gallery-state.json loaded");

    Ok(LoadedGallerySnapshot {
        snapshot,
        existed: true,
    })
}

pub async fn save(paths: &AppPaths, snapshot: &GallerySnapshot) -> Result<()> {
    let path = paths.gallery_state_path.clone();
    let json = serde_json::to_vec_pretty(snapshot)?;
    tokio::task::spawn_blocking(move || -> Result<()> {
        let mut file = AtomicWriteFile::options()
            .open(&path)
            .with_context(|| format!("opening atomic writer for {}", path.display()))?;
        file.write_all(&json)
            .with_context(|| format!("writing {}", path.display()))?;
        file.commit()
            .with_context(|| format!("committing {}", path.display()))?;
        Ok(())
    })
    .await??;

    tracing::info!(target: "sidecar", path = %paths.gallery_state_path.display(), "gallery-state.json saved atomically");
    Ok(())
}
