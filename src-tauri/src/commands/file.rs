use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::State;

use crate::{app_paths::AppPaths, gallery::event_log};

const MAX_TEXT_BYTES: u64 = 2 * 1024 * 1024;
const MAX_BINARY_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfAssetDiagnostic {
    pub artifact_id: String,
    pub pdf_url: String,
    pub local_file_path: String,
    pub local_file_exists: bool,
    pub is_file: bool,
    pub file_size_bytes: Option<u64>,
    pub canonical_file_path: Option<String>,
    pub data_dir: String,
    pub instance_dir: String,
    pub exe_path: Option<String>,
    pub exe_dir: Option<String>,
    pub under_data_dir: bool,
    pub under_instance_dir: bool,
    pub configured_asset_scope: String,
    pub error_message: Option<String>,
}

#[tauri::command]
pub async fn read_text_file(path: String, app_paths: State<'_, AppPaths>) -> Result<String, String> {
    tracing::debug!(target: "sidecar", path = %path, "read_text_file command");
    let (canonical, metadata) = validate_data_file(&path, &app_paths).await?;

    let bytes_to_read = metadata.len().min(MAX_TEXT_BYTES);
    let mut file = tokio::fs::File::open(&canonical)
        .await
        .map_err(|error| error.to_string())?;
    let mut buffer = vec![0; bytes_to_read as usize];
    use tokio::io::AsyncReadExt;
    file.read_exact(&mut buffer)
        .await
        .map_err(|error| error.to_string())?;

    let mut text = String::from_utf8_lossy(&buffer).to_string();
    if metadata.len() > MAX_TEXT_BYTES {
        text.push_str("\n\n--- 文件超过 2MB，已截断显示。---\n");
    }
    Ok(text)
}

#[tauri::command]
pub async fn read_binary_file(path: String, app_paths: State<'_, AppPaths>) -> Result<Vec<u8>, String> {
    tracing::debug!(target: "sidecar", path = %path, "read_binary_file command");
    let (canonical, metadata) = validate_data_file(&path, &app_paths).await?;
    if metadata.len() > MAX_BINARY_BYTES {
        return Err(format!("文件超过 {}MB，拒绝读取。", MAX_BINARY_BYTES / 1024 / 1024));
    }

    tokio::fs::read(&canonical)
        .await
        .map_err(|error| format!("读取 {} 失败: {error}", canonical.display()))
}

#[tauri::command]
pub async fn record_pdfjs_preview_requested(
    artifact_id: String,
    pdf_url: String,
    app_paths: State<'_, AppPaths>,
) -> Result<(), String> {
    tracing::info!(
        target: "sidecar",
        artifact_id = %artifact_id,
        pdf_url = %pdf_url,
        "PDF.js preview requested"
    );
    event_log::append_simple(
        &app_paths,
        "pdfjs_preview_requested",
        serde_json::json!({
            "artifactId": artifact_id,
            "pdfUrl": pdf_url,
        }),
    )
    .await
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn diagnose_pdf_asset_path(
    artifact_id: String,
    local_file_path: String,
    pdf_url: String,
    app_paths: State<'_, AppPaths>,
) -> Result<PdfAssetDiagnostic, String> {
    let requested = PathBuf::from(&local_file_path);
    let metadata = tokio::fs::metadata(&requested).await.ok();
    let canonical_file = requested.canonicalize().ok();
    let data_dir = app_paths.data_dir.canonicalize().unwrap_or_else(|_| app_paths.data_dir.clone());
    let instance_dir = app_paths
        .instance_dir
        .canonicalize()
        .unwrap_or_else(|_| app_paths.instance_dir.clone());
    let exe_path = std::env::current_exe().ok();
    let exe_dir = exe_path.as_ref().and_then(|path| path.parent()).map(Path::to_path_buf);
    let error_message = if metadata.is_none() {
        Some("PDF 本地文件不存在或不可访问。".to_string())
    } else if !canonical_file
        .as_ref()
        .map(|path| path.starts_with(&instance_dir))
        .unwrap_or(false)
    {
        Some("PDF 不在当前 portable instance 目录下，asset protocol 可能会拒绝。".to_string())
    } else {
        None
    };

    let diagnostic = PdfAssetDiagnostic {
        artifact_id,
        pdf_url,
        local_file_path,
        local_file_exists: metadata.is_some(),
        is_file: metadata.as_ref().map(|item| item.is_file()).unwrap_or(false),
        file_size_bytes: metadata.as_ref().map(|item| item.len()),
        canonical_file_path: canonical_file.as_ref().map(path_to_string),
        data_dir: path_to_string(&data_dir),
        instance_dir: path_to_string(&instance_dir),
        exe_path: exe_path.as_ref().map(path_to_string),
        exe_dir: exe_dir.as_ref().map(path_to_string),
        under_data_dir: canonical_file
            .as_ref()
            .map(|path| path.starts_with(&data_dir))
            .unwrap_or(false),
        under_instance_dir: canonical_file
            .as_ref()
            .map(|path| path.starts_with(&instance_dir))
            .unwrap_or(false),
        configured_asset_scope: "$EXE/../**".to_string(),
        error_message,
    };

    tracing::info!(
        target: "sidecar",
        artifact_id = %diagnostic.artifact_id,
        pdf_url = %diagnostic.pdf_url,
        local_file_path = %diagnostic.local_file_path,
        local_file_exists = diagnostic.local_file_exists,
        under_data_dir = diagnostic.under_data_dir,
        under_instance_dir = diagnostic.under_instance_dir,
        configured_asset_scope = %diagnostic.configured_asset_scope,
        "PDF asset diagnostic requested"
    );
    event_log::append_simple(
        &app_paths,
        "pdf_asset_diagnostic_requested",
        serde_json::json!(&diagnostic),
    )
    .await
    .map_err(|error| error.to_string())?;
    Ok(diagnostic)
}

fn canonicalize_existing(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize()
        .map_err(|error| format!("无法解析路径 {}: {error}", path.display()))
}

async fn validate_data_file(
    path: &str,
    app_paths: &AppPaths,
) -> Result<(PathBuf, std::fs::Metadata), String> {
    let requested = PathBuf::from(path);
    let data_dir = canonicalize_existing(&app_paths.data_dir)?;
    let canonical = canonicalize_existing(&requested)?;

    if !canonical.starts_with(&data_dir) {
        return Err("只能读取当前 Sidecar data 目录下的文件。".to_string());
    }

    let metadata = tokio::fs::metadata(&canonical)
        .await
        .map_err(|error| error.to_string())?;
    if !metadata.is_file() {
        return Err("目标路径不是文件。".to_string());
    }
    Ok((canonical, metadata))
}

fn path_to_string(path: impl AsRef<Path>) -> String {
    strip_windows_verbatim_prefix(path.as_ref().to_string_lossy().as_ref())
}

fn strip_windows_verbatim_prefix(path: &str) -> String {
    if let Some(rest) = path.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{rest}")
    } else if let Some(rest) = path.strip_prefix(r"\\?\") {
        rest.to_string()
    } else {
        path.to_string()
    }
}
