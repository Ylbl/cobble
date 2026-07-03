use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use directories_next::ProjectDirs;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub gallery_state_path: PathBuf,
    pub gallery_events_path: PathBuf,
    pub artifacts_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub debug_artifacts_dir: PathBuf,
}

impl AppPaths {
    pub fn init() -> Result<Self> {
        let project_dirs = ProjectDirs::from("", "", "ai-artifact-sidecar")
            .context("resolving system app data directory")?;
        let data_dir = project_dirs.data_local_dir().to_path_buf();
        let paths = Self {
            gallery_state_path: data_dir.join("gallery-state.json"),
            gallery_events_path: data_dir.join("gallery-events.jsonl"),
            artifacts_dir: data_dir.join("artifacts"),
            logs_dir: data_dir.join("logs"),
            debug_artifacts_dir: data_dir.join("debug-artifacts"),
            data_dir,
        };

        fs::create_dir_all(&paths.data_dir)
            .with_context(|| format!("creating {}", paths.data_dir.display()))?;
        fs::create_dir_all(&paths.artifacts_dir)
            .with_context(|| format!("creating {}", paths.artifacts_dir.display()))?;
        fs::create_dir_all(&paths.logs_dir)
            .with_context(|| format!("creating {}", paths.logs_dir.display()))?;
        fs::create_dir_all(&paths.debug_artifacts_dir)
            .with_context(|| format!("creating {}", paths.debug_artifacts_dir.display()))?;

        Ok(paths)
    }
}
