use std::{
    fs,
    net::SocketAddr,
    sync::Arc,
};

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    response::Response,
};
use http::HeaderName;
use http_body_util::BodyExt;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
};
use serde::Serialize;
use tauri::{Emitter, Manager};
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use tower::Service;

use crate::{app_paths::AppPaths, mcp::tools::SidecarMcpService, settings::state::ConfigState};

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

type McpService = StreamableHttpService<SidecarMcpService, LocalSessionManager>;

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
        "starting MCP Server on fixed configured port"
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
            tracing::error!(
                target: "sidecar",
                mcp_host = %host,
                mcp_port = port,
                error = %error,
                "MCP status = failed"
            );
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

    let mcp_service = StreamableHttpService::new(
        {
            let app = app.clone();
            move || Ok(SidecarMcpService::new(app.clone()))
        },
        Default::default(),
        StreamableHttpServerConfig::default()
            .with_sse_keep_alive(None)
            .with_cancellation_token(cancellation_token.child_token()),
    );

    let mcp_service = Arc::new(Mutex::new(mcp_service));

    // Wrapper that catches "Session not found" 404 and retries
    // without the stale session header, transparently auto-creating a new session.
    let router = axum::Router::new().fallback_service(
        tower::service_fn({
            let mcp_service = mcp_service.clone();
            move |req: Request<Body>| {
                let mcp_service = mcp_service.clone();
                async move { Ok(serve_mcp_with_recovery(mcp_service, req).await) }
            }
        }),
    );

    let status = McpServerStatus {
        running: true,
        status: McpRuntimeStatus::Running,
        host: host.clone(),
        url: Some(url.clone()),
        port: Some(port),
        error_message: None,
    };
    set_status(&app, status.clone()).await?;
    tracing::info!(target: "sidecar", mcp_url = %url, "MCP Server started");

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

const MCP_SESSION_ID: &str = "mcp-session-id";

/// Wraps the MCP service: on 404 "Session not found", strips the
/// Mcp-Session-Id header and retries, auto-creating a new session.
async fn serve_mcp_with_recovery(
    mcp_service: Arc<Mutex<McpService>>,
    req: Request<Body>,
) -> Response<Body> {
    let had_session = req
        .headers()
        .get(MCP_SESSION_ID)
        .map(|v| !v.is_empty())
        .unwrap_or(false);

    // Buffer the body so we can replay it on retry
    let (parts, body) = req.into_parts();
    let body_bytes = body
        .collect()
        .await
        .map(|collected| collected.to_bytes())
        .unwrap_or_default();

    let req1 = Request::from_parts(parts.clone(), Body::from(body_bytes.clone()));
    let resp = {
        let mut svc = mcp_service.lock().await;
        let raw = match svc.call(req1).await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!(target: "sidecar", ?e, "MCP service error");
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::empty())
                    .unwrap();
            }
        };
        mcp_response_to_body(raw).await
    };

    // If got 404 and had a session header, we need to:
    // 1. Create a new session via initialize
    // 2. Replay the original request with the new session ID
    if resp.status() == StatusCode::NOT_FOUND && had_session {
        let mut new_parts = parts.clone();
        new_parts.headers.remove(MCP_SESSION_ID);

        // Extract protocol version from original request headers (ZCode sends MCP-Protocol-Version)
        let proto_version = parts
            .headers
            .get("MCP-Protocol-Version")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("2025-11-25")
            .to_string();

        // Step 1: send initialize to create a proper MCP session
        let init_body = Body::from(
            serde_json::json!({
                "jsonrpc": "2.0",
                "method": "initialize",
                "params": {
                    "protocolVersion": proto_version,
                    "capabilities": {},
                    "clientInfo": { "name": "ZCode", "version": "1.0" }
                },
                "id": 0
            })
            .to_string(),
        );
        let mut init_req = Request::from_parts(new_parts.clone(), init_body);
        // Copy protocol version header to initialize request
        init_req.headers_mut().insert(
            http::HeaderName::from_static("mcp-protocol-version"),
            proto_version.parse().unwrap(),
        );
        let (init_session_id, mut new_parts) = {
            let mut svc = mcp_service.lock().await;
            let init_resp = match svc.call(init_req).await {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!(target: "sidecar", ?e, "MCP initialize failed during recovery");
                    return Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::empty())
                        .unwrap();
                }
            };
            // Extract session ID and consume response body (critical for rmcp state machine)
            let sid = init_resp
                .headers()
                .get(MCP_SESSION_ID)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());
            let (_parts, body) = init_resp.into_parts();
            let _ = body.collect().await; // consume body to complete the response cycle

            let mut p = new_parts;
            if let Some(ref sid) = sid {
                p.headers.insert(
                    HeaderName::from_static(MCP_SESSION_ID),
                    sid.parse().unwrap(),
                );
            }
            // Copy Accept header from original request
            if let Some(accept) = parts.headers.get("accept").cloned() {
                p.headers.insert("accept", accept);
            }
            // Copy protocol version header
            p.headers.insert(
                http::HeaderName::from_static("mcp-protocol-version"),
                proto_version.parse().unwrap(),
            );
            (sid, p)
        };

        let req2 = Request::from_parts(new_parts, Body::from(body_bytes));
        let raw = {
            let mut svc = mcp_service.lock().await;
            match svc.call(req2).await {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!(target: "sidecar", ?e, "MCP replay failed after recovery");
                    return Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::empty())
                        .unwrap();
                }
            }
        };
        return mcp_response_to_body(raw).await;
    }

    resp
}

/// Convert from `rmcp`'s `BoxBody` to axum's `Body`.
async fn mcp_response_to_body(
    resp: Response<http_body_util::combinators::BoxBody<axum::body::Bytes, std::convert::Infallible>>,
) -> Response<Body> {
    let (parts, body) = resp.into_parts();
    let bytes = body
        .collect()
        .await
        .map(|collected| collected.to_bytes())
        .unwrap_or_default();
    Response::from_parts(parts, Body::from(bytes))
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
