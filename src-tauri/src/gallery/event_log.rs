use anyhow::{Context, Result};
use chrono::Utc;
use serde::Serialize;
use serde_json::{json, Value};
use tokio::io::AsyncWriteExt;

use crate::app_paths::AppPaths;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryEvent {
    pub event_type: String,
    pub occurred_at: String,
    pub payload: Value,
}

impl GalleryEvent {
    pub fn new(event_type: impl Into<String>, payload: Value) -> Self {
        Self {
            event_type: event_type.into(),
            occurred_at: Utc::now().to_rfc3339(),
            payload,
        }
    }
}

pub async fn append(paths: &AppPaths, event: GalleryEvent) -> Result<()> {
    let path = paths.gallery_events_path.clone();
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await
        .with_context(|| format!("opening {}", path.display()))?;
    let line = serde_json::to_string(&event)?;
    file.write_all(line.as_bytes())
        .await
        .with_context(|| format!("writing {}", path.display()))?;
    file.write_all(b"\n")
        .await
        .with_context(|| format!("writing newline to {}", path.display()))?;
    file.flush()
        .await
        .with_context(|| format!("flushing {}", path.display()))?;
    tracing::debug!(target: "sidecar", event_type = %event.event_type, path = %path.display(), "gallery event appended");
    Ok(())
}

pub async fn append_simple(
    paths: &AppPaths,
    event_type: impl Into<String>,
    payload: Value,
) -> Result<()> {
    append(paths, GalleryEvent::new(event_type, payload)).await
}

pub async fn append_error(paths: &AppPaths, event_type: &str, error: &anyhow::Error) {
    if let Err(log_error) = append_simple(
        paths,
        event_type,
        json!({
            "error": error.to_string(),
        }),
    )
    .await
    {
        tracing::error!(target: "sidecar", ?log_error, "failed to append gallery error event");
    }
}
