use std::{fs, net::SocketAddr, sync::Arc};

use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
};
use serde::Serialize;
use tauri::{Emitter, Manager};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::{
    app_paths::AppPaths,
    mcp::{session_store::FileSessionStore, tools::SidecarMcpService},
    settings::state::ConfigState,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerStatus {
    pub running: bool,
    pub status: McpRuntimeStatus,
    pub host: String,
    pub url: Option<String>,
    pub port: Option<u16>,
    pub error_message: Option<String>,
}

impl Default for McpServerStatus {
    fn default() -> Self {
        Self {
            running: false,
            status: McpRuntimeStatus::Stopped,
            host: "127.0.0.1".to_string(),
            url: None,
            port: None,
            error_message: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum McpRuntimeStatus {
    Running,
    Stopped,
    Failed,
}

pub struct McpServerState {
    pub status: RwLock<McpServerStatus>,
    cancellation_token: RwLock<Option<CancellationToken>>,
}

impl Default for McpServerState {
    fn default() -> Self {
        Self {
            status: RwLock::new(McpServerStatus::default()),
            cancellation_token: RwLock::new(None),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct McpServerStatusFile<'a> {
    running: bool,
    status: &'a McpRuntimeStatus,
    transport: &'a str,
    url: Option<&'a str>,
    host: &'a str,
    port: Option<u16>,
    error_message: Option<&'a str>,
    pid: u32,
    version: &'a str,
}

pub async fn start(app: tauri::AppHandle) -> anyhow::Result<McpServerStatus> {
    start_or_restart(app, false).await
}

pub async fn restart(app: tauri::AppHandle) -> anyhow::Result<McpServerStatus> {
    tracing::info!(target: "sidecar", "MCP Server manual restart requested");
    start_or_restart(app, true).await
}

async fn start_or_restart(
    app: tauri::AppHandle,
    stop_existing: bool,
) -> anyhow::Result<McpServerStatus> {
    if stop_existing {
        stop_current(&app).await;
    }

    let config = app.state::<ConfigState>().get_config().await;
    let host = config.mcp.host.trim().to_string();
    let port = config.mcp.port;
    let url = format!("http://{host}:{port}/mcp");
    tracing::info!(
        target: "sidecar",
        mcp_host = %host,
        mcp_port = port,
        mcp_url = %url,
        "starting MCP Server"
    );

    let addr: SocketAddr = match format!("{host}:{port}").parse() {
        Ok(addr) => addr,
        Err(error) => {
            let status = failed_status(host, port, format!("Invalid MCP host/port: {error}"));
            set_status(&app, status.clone()).await?;
            return Ok(status);
        }
    };

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(error) => {
            let message = if error.kind() == std::io::ErrorKind::AddrInUse {
                format!("MCP Server 启动失败：端口 {port} 已被占用。请手动修改端口，然后点击 Restart MCP Server。")
            } else {
                format!("MCP Server 启动失败：{error}")
            };
            tracing::error!(target: "sidecar", mcp_host = %host, mcp_port = port, error = %error, "MCP status = failed");
            let status = failed_status(host, port, message);
            set_status(&app, status.clone()).await?;
            return Ok(status);
        }
    };

    let cancellation_token = CancellationToken::new();
    {
        let state = app.state::<McpServerState>();
        *state.cancellation_token.write().await = Some(cancellation_token.clone());
    }

    // Load persistent session store for cross-restart session recovery
    let app_paths = app.state::<AppPaths>();
    let session_store = Arc::new(
        FileSessionStore::load_from_path(app_paths.mcp_sessions_path.clone()).await?,
    );

    let mut config = StreamableHttpServerConfig::default();
    config.stateful_mode = true;
    config.session_store = Some(session_store);
    config.sse_keep_alive = None;
    config.cancellation_token = cancellation_token.child_token();

    let service = StreamableHttpService::new(
        {
            let app = app.clone();
            move || Ok(SidecarMcpService::new(app.clone()))
        },
        Arc::new(LocalSessionManager::default()),
        config,
    );

    let router = axum::Router::new().nest_service("/mcp", service);

    let status = McpServerStatus {
        running: true,
        status: McpRuntimeStatus::Running,
        host: host.clone(),
        url: Some(url.clone()),
        port: Some(port),
        error_message: None,
    };
    set_status(&app, status.clone()).await?;
    tracing::info!(target: "sidecar", mcp_url = %url, "MCP Server started with persistent session store");

    let app_for_task = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = axum::serve(listener, router)
            .with_graceful_shutdown(async move { cancellation_token.cancelled_owned().await })
            .await
        {
            tracing::error!(target: "sidecar", ?error, "MCP server stopped with error");
        }
        tracing::info!(target: "sidecar", "MCP Server stopped");
        let state = app_for_task.state::<McpServerState>();
        let mut status = state.status.write().await;
        if matches!(status.status, McpRuntimeStatus::Running) {
            status.running = false;
            status.status = McpRuntimeStatus::Stopped;
        }
    });

    Ok(status)
}

async fn stop_current(app: &tauri::AppHandle) {
    let state = app.state::<McpServerState>();
    let token = state.cancellation_token.write().await.take();
    if let Some(token) = token {
        tracing::info!(target: "sidecar", "stopping existing MCP Server");
        token.cancel();
    }
}

async fn set_status(app: &tauri::AppHandle, status: McpServerStatus) -> anyhow::Result<()> {
    {
        let state = app.state::<McpServerState>();
        *state.status.write().await = status.clone();
    }
    let app_paths = app.state::<AppPaths>();
    write_status_file(&app_paths, &status)?;
    if let Err(error) = app.emit("mcp-status-updated", status.clone()) {
        tracing::error!(target: "sidecar", ?error, "failed to emit mcp-status-updated");
    }
    Ok(())
}

fn failed_status(host: String, port: u16, message: String) -> McpServerStatus {
    McpServerStatus {
        running: false,
        status: McpRuntimeStatus::Failed,
        host,
        url: None,
        port: Some(port),
        error_message: Some(message),
    }
}

fn write_status_file(app_paths: &AppPaths, status: &McpServerStatus) -> anyhow::Result<()> {
    let path = app_paths.data_dir.join("mcp");
    fs::create_dir_all(&path)?;
    let status_file = McpServerStatusFile {
        running: status.running,
        status: &status.status,
        transport: "streamable-http",
        url: status.url.as_deref(),
        host: &status.host,
        port: status.port,
        error_message: status.error_message.as_deref(),
        pid: std::process::id(),
        version: env!("CARGO_PKG_VERSION"),
    };
    fs::write(
        path.join("mcp-server.json"),
        serde_json::to_string_pretty(&status_file)?,
    )?;
    Ok(())
}
