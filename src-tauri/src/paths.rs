use std::path::PathBuf;

pub fn debug_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")))
        .join(".sidecar-debug")
}

pub fn debug_logs_dir() -> PathBuf {
    debug_root().join("debug-logs")
}

pub fn debug_artifacts_dir() -> PathBuf {
    debug_root().join("debug-artifacts")
}

pub fn debug_mcp_dir() -> PathBuf {
    debug_root().join("debug-mcp")
}
