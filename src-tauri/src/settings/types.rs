use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::gallery::types::SidebarMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SidecarConfig {
    pub mcp: McpConfig,
    pub latex: LatexConfig,
    pub gallery: GalleryConfig,
}

impl Default for SidecarConfig {
    fn default() -> Self {
        Self {
            mcp: McpConfig {
                host: "127.0.0.1".to_string(),
                port: 39333,
            },
            latex: LatexConfig {
                engine: LatexEngine::Xelatex,
                compile_timeout_seconds: 60,
            },
            gallery: GalleryConfig {
                default_sidebar_mode: SidebarMode::Groups,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexConfig {
    pub engine: LatexEngine,
    pub compile_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryConfig {
    pub default_sidebar_mode: SidebarMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum LatexEngine {
    Pdflatex,
    Xelatex,
    Lualatex,
}

impl LatexEngine {
    pub fn latexmk_arg(&self) -> &'static str {
        match self {
            Self::Pdflatex => "-pdf",
            Self::Xelatex => "-xelatex",
            Self::Lualatex => "-lualatex",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SidecarConfigView {
    pub config: SidecarConfig,
    pub instance_folder_name: String,
    pub instance_dir: String,
    pub config_path: String,
    pub data_dir: String,
    pub gallery_state_path: String,
    pub gallery_events_path: String,
    pub artifacts_dir: String,
    pub logs_dir: String,
    pub debug_artifacts_dir: String,
    pub lock_path: String,
}
