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
                instructions: None,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

impl McpConfig {
    pub fn instructions(&self) -> &str {
        self.instructions
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(DEFAULT_MCP_INSTRUCTIONS)
    }
}

pub const DEFAULT_MCP_INSTRUCTIONS: &str = r#"Cobble 可以展示图片、SVG、PDF 和 LaTeX 编译结果。

## display_artifact_turn 用法

当用户要求展示图表、电路、化学结构、公式卡、LaTeX 图、TikZ 图或其他可视化内容时调用此工具。

一次助手回复应调用 display_artifact_turn 一次。
同一回复中的所有 artifact 放入 artifacts 数组。

## 支持的 artifact 类型

- "image": 需要 imageUrl
- "latex": 需要完整的 latexCode（可编译的完整 LaTeX 文档）
- "svg": 需要内联 svg 内容
- "pdf": 需要 pdfUrl

## LaTeX 最佳实践

推荐使用 standalone 文档类：
\documentclass[border=6pt,multi=false]{standalone}

多行内容或带图注时必须加 multi=false：
\documentclass[border=6pt,multi=false]{standalone}

带标题/图注的内容推荐用 varwidth 环境：
\documentclass[border=6pt,multi=false]{standalone}
\usepackage{varwidth}
\begin{document}
\begin{varwidth}{\linewidth}
...公式...
\\[12pt]
图1：标题说明
\end{varwidth}
\end{document}

## 会话管理

如果之前工具结果中有 sidecarSessionId，直接复用。
如果没有，提供清晰的 sessionTitle，不要自己编 sidecarSessionId。

Sidecar 使用配置的 LaTeX 引擎编译并展示 PDF 结果。"#;

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
    pub mcp_sessions_path: String,
}
