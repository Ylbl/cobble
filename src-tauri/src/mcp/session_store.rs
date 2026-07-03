use std::{collections::HashMap, path::PathBuf};

use async_trait::async_trait;
use rmcp::transport::streamable_http_server::session::store::{
    SessionState, SessionStore, SessionStoreError,
};
use tokio::sync::RwLock;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedMcpSessions {
    sessions: HashMap<String, SessionState>,
}

/// File-backed session store for MCP transport sessions.
///
/// Sessions are persisted to `<instance-dir>/data/mcp-sessions.json` so that
/// after a Sidecar restart, old `Mcp-Session-Id` values can be transparently
/// restored by rmcp without the client needing to re-initialize.
pub struct FileSessionStore {
    path: PathBuf,
    inner: RwLock<PersistedMcpSessions>,
}

impl FileSessionStore {
    pub async fn load_from_path(path: PathBuf) -> anyhow::Result<Self> {
        let sessions = match tokio::fs::read_to_string(&path).await {
            Ok(content) => {
                match serde_json::from_str::<PersistedMcpSessions>(&content) {
                    Ok(s) => {
                        tracing::info!(
                            target: "sidecar",
                            path = %path.display(),
                            count = s.sessions.len(),
                            "MCP session store loaded"
                        );
                        s
                    }
                    Err(error) => {
                        let ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let backup = path.with_file_name(format!(
                            "mcp-sessions.invalid.{}.json",
                            ts
                        ));
                        tokio::fs::rename(&path, &backup).await?;
                        tracing::error!(
                            target: "sidecar",
                            path = %path.display(),
                            backup = %backup.display(),
                            ?error,
                            "MCP session store corrupted, backed up and starting fresh"
                        );
                        PersistedMcpSessions { sessions: HashMap::new() }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::info!(
                    target: "sidecar",
                    path = %path.display(),
                    "MCP session store not found, starting fresh"
                );
                PersistedMcpSessions { sessions: HashMap::new() }
            }
            Err(error) => {
                tracing::error!(target: "sidecar", ?error, "failed to read MCP session store");
                anyhow::bail!(error);
            }
        };

        Ok(Self {
            path,
            inner: RwLock::new(sessions),
        })
    }

    async fn flush(&self, data: &PersistedMcpSessions) -> Result<(), SessionStoreError> {
        let json = serde_json::to_vec_pretty(data)?;
        // Atomic write via temp file + rename
        let tmp = self.path.with_extension("tmp");
        tokio::fs::write(&tmp, &json).await?;
        tokio::fs::rename(&tmp, &self.path).await?;
        Ok(())
    }
}

#[async_trait]
impl SessionStore for FileSessionStore {
    async fn load(&self, session_id: &str) -> Result<Option<SessionState>, SessionStoreError> {
        let guard = self.inner.read().await;
        let state = guard.sessions.get(session_id).cloned();
        tracing::debug!(
            target: "sidecar",
            %session_id,
            hit = state.is_some(),
            "MCP session store load"
        );
        Ok(state)
    }

    async fn store(&self, session_id: &str, state: &SessionState) -> Result<(), SessionStoreError> {
        let snapshot = {
            let mut guard = self.inner.write().await;
            guard.sessions.insert(session_id.to_owned(), state.clone());
            guard.clone()
        };
        tracing::debug!(target: "sidecar", %session_id, "MCP session store save");
        self.flush(&snapshot).await?;
        Ok(())
    }

    async fn delete(&self, session_id: &str) -> Result<(), SessionStoreError> {
        let snapshot = {
            let mut guard = self.inner.write().await;
            guard.sessions.remove(session_id);
            guard.clone()
        };
        tracing::debug!(target: "sidecar", %session_id, "MCP session store delete");
        self.flush(&snapshot).await?;
        Ok(())
    }
}
