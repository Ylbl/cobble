use serde::{Deserialize, Serialize};

use crate::settings::types::LatexEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexToolStatus {
    pub name: String,
    pub status: ToolStatus,
    pub path: Option<String>,
    pub version: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolStatus {
    Found,
    Missing,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexPackageStatus {
    pub name: String,
    pub status: PackageStatus,
    pub path: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackageStatus {
    Found,
    Missing,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexDoctorReport {
    pub checked_at: String,
    pub default_engine: LatexEngine,
    pub tools: Vec<LatexToolStatus>,
    pub packages: Vec<LatexPackageStatus>,
    pub smoke_test: Option<LatexCompileResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexCompileResult {
    pub ok: bool,
    pub artifact_id: String,
    pub engine: LatexEngine,
    pub elapsed_ms: u128,
    pub work_dir: String,
    pub main_tex_path: String,
    pub source_file_path: String,
    pub pdf_path: Option<String>,
    pub pdf_file_path: Option<String>,
    pub log_path: Option<String>,
    pub log_file_path: Option<String>,
    pub stdout_path: String,
    pub stderr_path: String,
    pub exit_code: Option<i32>,
    pub error_message: Option<String>,
    pub finished_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexCompileRequest {
    pub artifact_id: String,
    pub title: String,
    pub latex_code: String,
    pub engine: LatexEngine,
    pub work_dir: String,
    pub timeout_seconds: u64,
}
