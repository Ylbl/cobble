use anyhow::Result;
use tokio::sync::RwLock;

use crate::{
    app_paths::AppPaths,
    settings::{
        persistence,
        types::{SidecarConfig, SidecarConfigView},
    },
};

pub struct ConfigState {
    paths: AppPaths,
    config: RwLock<SidecarConfig>,
}

impl ConfigState {
    pub fn new(paths: AppPaths, config: SidecarConfig) -> Self {
        tracing::info!(
            target: "sidecar",
            instance_name = %config.instance_name,
            instance_dir = %paths.instance_dir.display(),
            config_path = %paths.config_path.display(),
            data_dir = %paths.data_dir.display(),
            mcp_host = %config.mcp.host,
            mcp_port = config.mcp.port,
            latex_engine = ?config.latex.engine,
            "config state initialized"
        );
        Self {
            paths,
            config: RwLock::new(config),
        }
    }

    pub async fn get_config(&self) -> SidecarConfig {
        self.config.read().await.clone()
    }

    pub async fn get_config_view(&self) -> SidecarConfigView {
        self.paths.to_config_view(self.get_config().await)
    }

    pub async fn update_config(&self, config: SidecarConfig) -> Result<SidecarConfigView> {
        tracing::info!(
            target: "sidecar",
            instance_name = %config.instance_name,
            mcp_host = %config.mcp.host,
            mcp_port = config.mcp.port,
            data_dir = %config.paths.data_dir,
            "saving sidecar config"
        );
        persistence::save_config_atomic(&self.paths.config_path, &config).await?;
        *self.config.write().await = config;
        Ok(self.get_config_view().await)
    }
}
