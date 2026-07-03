use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use fs2::FileExt;

use crate::settings::types::{SidecarConfig, SidecarConfigView};

#[derive(Debug)]
pub struct InstanceLock {
    _file: File,
}

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub instance_dir: PathBuf,
    pub config_path: PathBuf,
    pub data_dir: PathBuf,
    pub gallery_state_path: PathBuf,
    pub gallery_events_path: PathBuf,
    pub artifacts_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub debug_artifacts_dir: PathBuf,
    pub lock_path: PathBuf,
}

impl AppPaths {
    pub fn bootstrap() -> Result<(Self, SidecarConfig, InstanceLock)> {
        let instance_dir = resolve_instance_dir()?;
        fs::create_dir_all(&instance_dir)
            .with_context(|| format!("creating {}", instance_dir.display()))?;
        let config_path = instance_dir.join("sidecar.config.json");
        let config = crate::settings::persistence::load_or_create_config(&config_path)?;
        let paths = Self::from_config(instance_dir, config_path, &config)?;
        paths.create_dirs()?;
        let lock = paths.acquire_lock()?;
        Ok((paths, config, lock))
    }

    pub fn from_config(
        instance_dir: PathBuf,
        config_path: PathBuf,
        config: &SidecarConfig,
    ) -> Result<Self> {
        let data_dir = resolve_data_dir(&instance_dir, &config.paths.data_dir);
        Ok(Self {
            gallery_state_path: data_dir.join("gallery-state.json"),
            gallery_events_path: data_dir.join("gallery-events.jsonl"),
            artifacts_dir: data_dir.join("artifacts"),
            logs_dir: data_dir.join("logs"),
            debug_artifacts_dir: data_dir.join("debug-artifacts"),
            lock_path: data_dir.join(".sidecar.lock"),
            data_dir,
            instance_dir,
            config_path,
        })
    }

    pub fn create_dirs(&self) -> Result<()> {
        fs::create_dir_all(&self.data_dir)
            .with_context(|| format!("creating {}", self.data_dir.display()))?;
        fs::create_dir_all(&self.artifacts_dir)
            .with_context(|| format!("creating {}", self.artifacts_dir.display()))?;
        fs::create_dir_all(&self.logs_dir)
            .with_context(|| format!("creating {}", self.logs_dir.display()))?;
        fs::create_dir_all(&self.debug_artifacts_dir)
            .with_context(|| format!("creating {}", self.debug_artifacts_dir.display()))?;
        Ok(())
    }

    pub fn acquire_lock(&self) -> Result<InstanceLock> {
        let file = File::options()
            .create(true)
            .read(true)
            .write(true)
            .open(&self.lock_path)
            .with_context(|| format!("opening lock file {}", self.lock_path.display()))?;
        file.try_lock_exclusive().with_context(|| {
            "当前 Sidecar 实例已经在运行。如果要多开，请复制整个 Sidecar 文件夹，并修改新文件夹里的 sidecar.config.json 端口。"
        })?;
        tracing::info!(target: "sidecar", lock_path = %self.lock_path.display(), "instance lock acquired");
        Ok(InstanceLock { _file: file })
    }

    pub fn to_config_view(&self, config: SidecarConfig) -> SidecarConfigView {
        SidecarConfigView {
            config,
            instance_dir: path_to_string(&self.instance_dir),
            config_path: path_to_string(&self.config_path),
            data_dir: path_to_string(&self.data_dir),
            gallery_state_path: path_to_string(&self.gallery_state_path),
            gallery_events_path: path_to_string(&self.gallery_events_path),
            artifacts_dir: path_to_string(&self.artifacts_dir),
            logs_dir: path_to_string(&self.logs_dir),
            debug_artifacts_dir: path_to_string(&self.debug_artifacts_dir),
            lock_path: path_to_string(&self.lock_path),
        }
    }
}

fn resolve_instance_dir() -> Result<PathBuf> {
    if let Ok(value) = std::env::var("SIDECAR_INSTANCE_DIR") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed)
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(trimmed)));
        }
    }

    let exe = std::env::current_exe().context("resolving current exe path")?;
    exe.parent()
        .map(Path::to_path_buf)
        .context("current exe has no parent directory")
}

fn resolve_data_dir(instance_dir: &Path, configured: &str) -> PathBuf {
    let raw = PathBuf::from(configured.trim());
    if raw.is_absolute() {
        raw
    } else {
        instance_dir.join(raw)
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
