use std::time::Duration;

use tokio::{process::Command, time::timeout};

use crate::latex::types::{LatexPackageStatus, PackageStatus};

const PACKAGES: &[&str] = &[
    "standalone.cls",
    "tikz.sty",
    "pgfplots.sty",
    "circuitikz.sty",
    "chemfig.sty",
    "mhchem.sty",
    "amsmath.sty",
    "fontspec.sty",
    "xeCJK.sty",
    "graphicx.sty",
];

pub async fn detect_packages() -> Vec<LatexPackageStatus> {
    tracing::info!(target: "sidecar", "LaTeX package detection started");
    let kpsewhich = match which::which("kpsewhich") {
        Ok(path) => path,
        Err(_) => {
            return PACKAGES
                .iter()
                .map(|name| LatexPackageStatus {
                    name: (*name).to_string(),
                    status: PackageStatus::Failed,
                    path: None,
                    error_message: Some("kpsewhich not found".to_string()),
                })
                .collect();
        }
    };

    let mut results = Vec::new();
    for package in PACKAGES {
        let output = timeout(
            Duration::from_secs(5),
            Command::new(&kpsewhich).arg(package).output(),
        )
        .await;
        results.push(match output {
            Ok(Ok(output)) if output.status.success() => LatexPackageStatus {
                name: (*package).to_string(),
                status: PackageStatus::Found,
                path: Some(String::from_utf8_lossy(&output.stdout).trim().to_string()),
                error_message: None,
            },
            Ok(Ok(output)) => LatexPackageStatus {
                name: (*package).to_string(),
                status: PackageStatus::Missing,
                path: None,
                error_message: Some(String::from_utf8_lossy(&output.stderr).trim().to_string()),
            },
            Ok(Err(error)) => LatexPackageStatus {
                name: (*package).to_string(),
                status: PackageStatus::Failed,
                path: None,
                error_message: Some(error.to_string()),
            },
            Err(_) => LatexPackageStatus {
                name: (*package).to_string(),
                status: PackageStatus::Failed,
                path: None,
                error_message: Some("kpsewhich timed out".to_string()),
            },
        });
    }
    tracing::info!(target: "sidecar", count = results.len(), "LaTeX package detection completed");
    results
}
