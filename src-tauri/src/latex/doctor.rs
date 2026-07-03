use chrono::Utc;

use crate::{
    latex::{
        detector, package_detector,
        types::{LatexCompileResult, LatexDoctorReport},
    },
    settings::types::{LatexEngine, SidecarConfig},
};

pub async fn environment_report(config: &SidecarConfig) -> LatexDoctorReport {
    let tools = detector::detect_tools().await;
    let packages = package_detector::detect_packages().await;
    tracing::info!(target: "sidecar", "LaTeX doctor report generated");
    LatexDoctorReport {
        checked_at: Utc::now().to_rfc3339(),
        default_engine: config.latex.engine.clone(),
        tools,
        packages,
        smoke_test: None,
    }
}

pub fn report_with_smoke_test(
    engine: LatexEngine,
    smoke_test: LatexCompileResult,
) -> LatexDoctorReport {
    LatexDoctorReport {
        checked_at: Utc::now().to_rfc3339(),
        default_engine: engine,
        tools: Vec::new(),
        packages: Vec::new(),
        smoke_test: Some(smoke_test),
    }
}
