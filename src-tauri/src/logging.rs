use std::env;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

use crate::app_paths::AppPaths;

pub struct LoggingGuard {
    _file_guard: WorkerGuard,
}

pub fn init(app_paths: &AppPaths) -> LoggingGuard {
    let console_filter =
        env::var("SIDECAR_CONSOLE_LOG").unwrap_or_else(|_| "info,sidecar=debug".to_string());
    let file_filter = env::var("SIDECAR_FILE_LOG")
        .unwrap_or_else(|_| "debug,sidecar=trace,rmcp=debug".to_string());

    let log_dir = app_paths.logs_dir.clone();
    let file_appender = tracing_appender::rolling::daily(log_dir, "app.log");
    let (file_writer, file_guard) = tracing_appender::non_blocking(file_appender);

    let console_layer = fmt::layer()
        .compact()
        .with_target(true)
        .with_filter(EnvFilter::new(console_filter));

    let file_layer = fmt::layer()
        .json()
        .with_writer(file_writer)
        .with_filter(EnvFilter::new(file_filter));

    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .init();

    tracing::info!(target: "sidecar", "logging initialized");

    LoggingGuard {
        _file_guard: file_guard,
    }
}
