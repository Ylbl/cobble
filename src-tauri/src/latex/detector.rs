use std::{path::PathBuf, time::Duration};

use tokio::{process::Command, time::timeout};

use crate::latex::types::{LatexToolStatus, ToolStatus};

const TOOLS: &[&str] = &["latexmk", "xelatex", "pdflatex", "lualatex", "kpsewhich"];

pub async fn detect_tools() -> Vec<LatexToolStatus> {
    tracing::info!(target: "sidecar", "LaTeX tool detection started");
    let mut results = Vec::new();
    for tool in TOOLS {
        results.push(detect_tool(tool).await);
    }
    tracing::info!(target: "sidecar", count = results.len(), "LaTeX tool detection completed");
    results
}

async fn detect_tool(name: &str) -> LatexToolStatus {
    match which::which(name) {
        Ok(path) => version_status(name, path).await,
        Err(error) => LatexToolStatus {
            name: name.to_string(),
            status: ToolStatus::Missing,
            path: None,
            version: None,
            error_message: Some(error.to_string()),
        },
    }
}

async fn version_status(name: &str, path: PathBuf) -> LatexToolStatus {
    let output = timeout(
        Duration::from_secs(5),
        Command::new(&path).arg("--version").output(),
    )
    .await;
    match output {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let version = stdout
                .lines()
                .next()
                .or_else(|| stderr.lines().next())
                .map(|line| line.trim().to_string());
            LatexToolStatus {
                name: name.to_string(),
                status: if output.status.success() {
                    ToolStatus::Found
                } else {
                    ToolStatus::Failed
                },
                path: Some(path.to_string_lossy().to_string()),
                version,
                error_message: if output.status.success() {
                    None
                } else {
                    Some(stderr.to_string())
                },
            }
        }
        Ok(Err(error)) => LatexToolStatus {
            name: name.to_string(),
            status: ToolStatus::Failed,
            path: Some(path.to_string_lossy().to_string()),
            version: None,
            error_message: Some(error.to_string()),
        },
        Err(_) => LatexToolStatus {
            name: name.to_string(),
            status: ToolStatus::Failed,
            path: Some(path.to_string_lossy().to_string()),
            version: None,
            error_message: Some("version command timed out".to_string()),
        },
    }
}
