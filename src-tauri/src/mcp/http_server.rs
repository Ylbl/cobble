use std::{fs, net::SocketAddr};

use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
};
use serde::Serialize;
use tauri::Manager;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::{mcp::tools::SidecarMcpService, paths};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerStatus {
    pub running: bool,
    pub url: Option<String>,
    pub port: Option<u16>,
}

impl Default for McpServerStatus {
    fn default() -> Self {
        Self {
            running: false,
            url: None,
            port: None,
        }
    }
}

pub struct McpServerState {
    pub status: RwLock<McpServerStatus>,
}

impl Default for McpServerState {
    fn default() -> Self {
        Self {
            status: RwLock::new(McpServerStatus::default()),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct McpServerStatusFile<'a> {
    running: bool,
    transport: &'a str,
    url: &'a str,
    pid: u32,
    version: &'a str,
}

pub async fn start(app: tauri::AppHandle) -> anyhow::Result<()> {
    tracing::info!(target: "sidecar", "starting MCP Streamable HTTP server");

    let listener = bind_first_available().await?;
    let addr = listener.local_addr()?;
    let url = format!("http://{addr}/mcp");
    let port = addr.port();
    let cancellation_token = CancellationToken::new();

    let service: StreamableHttpService<SidecarMcpService, LocalSessionManager> =
        StreamableHttpService::new(
            {
                let app = app.clone();
                move || Ok(SidecarMcpService::new(app.clone()))
            },
            Default::default(),
            StreamableHttpServerConfig::default()
                .with_sse_keep_alive(None)
                .with_cancellation_token(cancellation_token.child_token()),
        );

    let router = axum::Router::new().nest_service("/mcp", service);

    {
        let state = app.state::<McpServerState>();
        *state.status.write().await = McpServerStatus {
            running: true,
            url: Some(url.clone()),
            port: Some(port),
        };
    }

    write_status_file(&url)?;
    tracing::info!(target: "sidecar", url = %url, "MCP Server started");

    axum::serve(listener, router)
        .with_graceful_shutdown(async move { cancellation_token.cancelled_owned().await })
        .await?;

    Ok(())
}

async fn bind_first_available() -> anyhow::Result<tokio::net::TcpListener> {
    for port in 39333..=39336 {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => return Ok(listener),
            Err(error) => {
                tracing::warn!(target: "sidecar", port, ?error, "MCP port unavailable");
            }
        }
    }

    anyhow::bail!("No MCP server port available in 39333..=39336")
}

fn write_status_file(url: &str) -> anyhow::Result<()> {
    let path = paths::debug_mcp_dir();
    fs::create_dir_all(&path)?;
    let status = McpServerStatusFile {
        running: true,
        transport: "streamable-http",
        url,
        pid: std::process::id(),
        version: env!("CARGO_PKG_VERSION"),
    };
    fs::write(
        path.join("mcp-server.json"),
        serde_json::to_string_pretty(&status)?,
    )?;
    Ok(())
}
