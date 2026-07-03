use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use chrono::Utc;
use tokio::{process::Command, time::timeout};
use uuid::Uuid;

use crate::{
    app_paths::AppPaths,
    latex::types::{LatexCompileRequest, LatexCompileResult},
    settings::types::LatexEngine,
};

const SMOKE_TEX: &str = r#"\documentclass[tikz,border=6pt]{standalone}
\begin{document}
\begin{tikzpicture}
  \draw[thick] (0,0) circle (1);
  \draw[->] (0,0) -- (1.5,0);
  \node at (0,-1.4) {LaTeX compile smoke test};
\end{tikzpicture}
\end{document}
"#;

pub async fn run_smoke_test(
    paths: &AppPaths,
    engine: LatexEngine,
    timeout_seconds: u64,
) -> Result<LatexCompileResult> {
    let artifact_id = format!("art_{}", Uuid::new_v4());
    let work_dir = paths.artifacts_dir.join(&artifact_id);
    compile_latex_artifact(LatexCompileRequest {
        artifact_id,
        title: "LaTeX Smoke Test".to_string(),
        latex_code: SMOKE_TEX.to_string(),
        engine,
        work_dir: path_to_string(&work_dir),
        timeout_seconds,
    })
    .await
}

pub async fn compile_latex_artifact(
    request: LatexCompileRequest,
) -> Result<LatexCompileResult> {
    let started = Instant::now();
    let work_dir = PathBuf::from(&request.work_dir);
    let out_dir = work_dir.join("out");
    let main_tex_path = work_dir.join("main.tex");
    let stdout_path = work_dir.join("stdout.txt");
    let stderr_path = work_dir.join("stderr.txt");
    let pdf_path = out_dir.join("main.pdf");
    let log_path = out_dir.join("main.log");

    tokio::fs::create_dir_all(&out_dir)
        .await
        .with_context(|| format!("creating {}", out_dir.display()))?;
    tokio::fs::write(&main_tex_path, request.latex_code.as_bytes())
        .await
        .with_context(|| format!("writing {}", main_tex_path.display()))?;

    tracing::info!(
        target: "sidecar",
        artifact_id = %request.artifact_id,
        title = %request.title,
        latex_chars = request.latex_code.chars().count(),
        engine = ?request.engine,
        work_dir = %work_dir.display(),
        main_tex_path = %main_tex_path.display(),
        "LaTeX artifact source written"
    );

    let latexmk = match which::which("latexmk") {
        Ok(path) => path,
        Err(error) => {
            let message = format!("latexmk not found: {error}");
            write_text(&stdout_path, "").await?;
            write_text(&stderr_path, &message).await?;
            return Ok(result(
                &request,
                started.elapsed().as_millis(),
                false,
                work_dir,
                main_tex_path,
                None,
                log_path.exists().then_some(log_path),
                stdout_path,
                stderr_path,
                None,
                Some(message),
            ));
        }
    };

    let command_args = vec![
        request.engine.latexmk_arg().to_string(),
        "-interaction=nonstopmode".to_string(),
        "-halt-on-error".to_string(),
        "-file-line-error".to_string(),
        "-outdir=out".to_string(),
        "main.tex".to_string(),
    ];

    tracing::info!(
        target: "sidecar",
        artifact_id = %request.artifact_id,
        latexmk = %latexmk.display(),
        args = ?command_args,
        working_dir = %work_dir.display(),
        timeout_seconds = request.timeout_seconds,
        "running latexmk for artifact"
    );

    let mut command = Command::new(&latexmk);
    command.args(&command_args).current_dir(&work_dir);
    let output = timeout(Duration::from_secs(request.timeout_seconds), command.output()).await;

    match output {
        Ok(Ok(output)) => {
            tokio::fs::write(&stdout_path, &output.stdout).await?;
            tokio::fs::write(&stderr_path, &output.stderr).await?;
            let pdf_exists = pdf_path.exists();
            let log_exists = log_path.exists();
            let ok = output.status.success() && pdf_exists;
            let error_message = if ok {
                None
            } else if output.status.success() {
                Some("latexmk completed but out/main.pdf was not generated".to_string())
            } else {
                Some("latexmk exited with failure".to_string())
            };

            tracing::info!(
                target: "sidecar",
                artifact_id = %request.artifact_id,
                exit_code = output.status.code(),
                pdf_exists,
                log_exists,
                stdout_path = %stdout_path.display(),
                stderr_path = %stderr_path.display(),
                log_path = %log_path.display(),
                elapsed_ms = started.elapsed().as_millis(),
                "latexmk artifact compile completed"
            );

            Ok(result(
                &request,
                started.elapsed().as_millis(),
                ok,
                work_dir,
                main_tex_path,
                pdf_exists.then_some(pdf_path),
                log_exists.then_some(log_path),
                stdout_path,
                stderr_path,
                output.status.code(),
                error_message,
            ))
        }
        Ok(Err(error)) => {
            write_text(&stdout_path, "").await?;
            write_text(&stderr_path, &error.to_string()).await?;
            tracing::error!(target: "sidecar", artifact_id = %request.artifact_id, ?error, "failed to execute latexmk");
            Ok(result(
                &request,
                started.elapsed().as_millis(),
                false,
                work_dir,
                main_tex_path,
                None,
                log_path.exists().then_some(log_path),
                stdout_path,
                stderr_path,
                None,
                Some(error.to_string()),
            ))
        }
        Err(_) => {
            let message = format!("latexmk timed out after {} seconds", request.timeout_seconds);
            write_text(&stdout_path, "").await?;
            write_text(&stderr_path, &message).await?;
            tracing::error!(target: "sidecar", artifact_id = %request.artifact_id, timeout_seconds = request.timeout_seconds, "latexmk timed out");
            Ok(result(
                &request,
                started.elapsed().as_millis(),
                false,
                work_dir,
                main_tex_path,
                None,
                log_path.exists().then_some(log_path),
                stdout_path,
                stderr_path,
                None,
                Some(message),
            ))
        }
    }
}

async fn write_text(path: &Path, value: &str) -> Result<()> {
    tokio::fs::write(path, value)
        .await
        .with_context(|| format!("writing {}", path.display()))
}

#[allow(clippy::too_many_arguments)]
fn result(
    request: &LatexCompileRequest,
    elapsed_ms: u128,
    ok: bool,
    work_dir: PathBuf,
    main_tex_path: PathBuf,
    pdf_path: Option<PathBuf>,
    log_path: Option<PathBuf>,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
    exit_code: Option<i32>,
    error_message: Option<String>,
) -> LatexCompileResult {
    let source_file_path = path_to_string(&main_tex_path);
    let pdf_file_path = pdf_path.as_ref().map(path_to_string);
    let log_file_path = log_path.as_ref().map(path_to_string);
    LatexCompileResult {
        ok,
        artifact_id: request.artifact_id.clone(),
        engine: request.engine.clone(),
        elapsed_ms,
        work_dir: path_to_string(&work_dir),
        main_tex_path: source_file_path.clone(),
        source_file_path,
        pdf_path: pdf_file_path.clone(),
        pdf_file_path,
        log_path: log_file_path.clone(),
        log_file_path,
        stdout_path: path_to_string(&stdout_path),
        stderr_path: path_to_string(&stderr_path),
        exit_code,
        error_message,
        finished_at: Utc::now().to_rfc3339(),
    }
}

fn path_to_string(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    strip_windows_verbatim_prefix(
        path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .as_ref(),
    )
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
