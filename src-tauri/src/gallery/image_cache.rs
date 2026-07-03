use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use tokio::io::AsyncWriteExt;

use crate::app_paths::AppPaths;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageDownloadPlan {
    pub artifact_id: String,
    pub image_url: String,
    pub artifact_dir: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageDownloadResult {
    pub ok: bool,
    pub status_code: Option<u16>,
    pub content_type: Option<String>,
    pub local_file_path: Option<String>,
    pub file_extension: Option<String>,
    pub error_message: Option<String>,
}

pub async fn cache_remote_image(
    paths: &AppPaths,
    artifact_id: &str,
    image_url: &str,
) -> ImageDownloadResult {
    let plan = ImageDownloadPlan {
        artifact_id: artifact_id.to_string(),
        image_url: image_url.to_string(),
        artifact_dir: paths.artifacts_dir.join(artifact_id).display().to_string(),
    };
    tracing::info!(
        target: "sidecar",
        artifact_id,
        image_url,
        artifact_dir = %plan.artifact_dir,
        "image download started"
    );

    match cache_remote_image_inner(paths, artifact_id, image_url).await {
        Ok(result) => result,
        Err(error) => {
            tracing::error!(target: "sidecar", artifact_id, error = %error, "image download failed");
            ImageDownloadResult {
                ok: false,
                status_code: None,
                content_type: None,
                local_file_path: None,
                file_extension: None,
                error_message: Some(error.to_string()),
            }
        }
    }
}

async fn cache_remote_image_inner(
    paths: &AppPaths,
    artifact_id: &str,
    image_url: &str,
) -> Result<ImageDownloadResult> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .context("building image download client")?;
    let response = client
        .get(image_url)
        .send()
        .await
        .with_context(|| format!("downloading image for artifact {artifact_id}"))?;
    let status = response.status();
    let status_code = status.as_u16();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());

    tracing::info!(
        target: "sidecar",
        artifact_id,
        status_code,
        content_type = content_type.as_deref().unwrap_or(""),
        "image download response received"
    );

    if !status.is_success() {
        return Err(anyhow!("image download returned HTTP {status_code}"));
    }

    let bytes = response
        .bytes()
        .await
        .context("reading image response body")?;
    let extension = extension_from_content_type(content_type.as_deref())
        .or_else(|| extension_from_url(image_url))
        .unwrap_or_else(|| "bin".to_string());
    let file_name = sanitize_filename::sanitize(format!("original-image.{extension}"));
    let artifact_dir = paths
        .artifacts_dir
        .join(sanitize_filename::sanitize(artifact_id));
    tokio::fs::create_dir_all(&artifact_dir)
        .await
        .with_context(|| format!("creating {}", artifact_dir.display()))?;
    let local_path = artifact_dir.join(file_name);
    let mut file = tokio::fs::File::create(&local_path)
        .await
        .with_context(|| format!("creating {}", local_path.display()))?;
    file.write_all(&bytes)
        .await
        .with_context(|| format!("writing {}", local_path.display()))?;
    file.flush()
        .await
        .with_context(|| format!("flushing {}", local_path.display()))?;

    tracing::info!(
        target: "sidecar",
        artifact_id,
        path = %local_path.display(),
        bytes = bytes.len(),
        "image saved to local cache"
    );

    Ok(ImageDownloadResult {
        ok: true,
        status_code: Some(status_code),
        content_type,
        local_file_path: Some(path_to_string(&local_path)),
        file_extension: Some(extension),
        error_message: None,
    })
}

pub async fn write_artifact_manifest<T: Serialize>(
    paths: &AppPaths,
    artifact_id: &str,
    value: &T,
) -> Result<()> {
    let artifact_dir = paths
        .artifacts_dir
        .join(sanitize_filename::sanitize(artifact_id));
    tokio::fs::create_dir_all(&artifact_dir)
        .await
        .with_context(|| format!("creating {}", artifact_dir.display()))?;
    let path = artifact_dir.join("artifact.json");
    let json = serde_json::to_vec_pretty(value)?;
    tokio::fs::write(&path, json)
        .await
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

fn extension_from_content_type(content_type: Option<&str>) -> Option<String> {
    let parsed: mime::Mime = content_type?.parse().ok()?;
    match (parsed.type_(), parsed.subtype().as_str()) {
        (mime::IMAGE, "png") => Some("png".to_string()),
        (mime::IMAGE, "jpeg") => Some("jpg".to_string()),
        (mime::IMAGE, "gif") => Some("gif".to_string()),
        (mime::IMAGE, "webp") => Some("webp".to_string()),
        (mime::IMAGE, "svg+xml") => Some("svg".to_string()),
        _ => None,
    }
}

fn extension_from_url(image_url: &str) -> Option<String> {
    let parsed = reqwest::Url::parse(image_url).ok()?;
    extension_from_path(Path::new(parsed.path()))
}

fn extension_from_path(path: &Path) -> Option<String> {
    let extension = path.extension()?.to_string_lossy().to_ascii_lowercase();
    match extension.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" => Some(extension),
        _ => None,
    }
}

fn path_to_string(path: &PathBuf) -> String {
    path.to_string_lossy().to_string()
}
