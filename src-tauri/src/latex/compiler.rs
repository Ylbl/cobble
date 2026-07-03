use std::{path::PathBuf, time::Duration};

use anyhow::{Context, Result};
use chrono::Utc;
use tokio::{process::Command, time::timeout};
use uuid::Uuid;

use crate::{app_paths::AppPaths, latex::types::LatexCompileResult, settings::types::LatexEngine};

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
    tokio::fs::create_dir_all(&work_dir)
        .await
        .with_context(|| format!("creating {}", work_dir.display()))?;

    let main_tex_path = work_dir.join("main.tex");
    let stdout_path = work_dir.join("stdout.txt");
    let stderr_path = work_dir.join("stderr.txt");
    let pdf_path = work_dir.join("main.pdf");
    let log_path = work_dir.join("main.log");

    tracing::info!(
        target: "sidecar",
        work_dir = %work_dir.display(),
        engine = ?engine,
        "LaTeX smoke test started"
    );
    tokio::fs::write(&main_tex_path, SMOKE_TEX)
        .await
        .with_context(|| format!("writing {}", main_tex_path.display()))?;

    let latexmk = match which::which("latexmk") {
        Ok(path) => path,
        Err(error) => {
            let message = format!("latexmk not found: {error}");
            write_text(&stdout_path, "").await?;
            write_text(&stderr_path, &message).await?;
            return Ok(result(
                false,
                engine,
                work_dir,
                main_tex_path,
                None,
                None,
                stdout_path,
                stderr_path,
                None,
                Some(message),
            ));
        }
    };

    let command_description = format!(
        "{} {} -interaction=nonstopmode -halt-on-error -file-line-error main.tex",
        latexmk.display(),
        engine.latexmk_arg()
    );
    tracing::info!(target: "sidecar", command = %command_description, "running latexmk");
    let output = timeout(
        Duration::from_secs(timeout_seconds),
        Command::new(latexmk)
            .arg(engine.latexmk_arg())
            .arg("-interaction=nonstopmode")
            .arg("-halt-on-error")
            .arg("-file-line-error")
            .arg("main.tex")
            .current_dir(&work_dir)
            .output(),
    )
    .await;

    match output {
        Ok(Ok(output)) => {
            tokio::fs::write(&stdout_path, &output.stdout).await?;
            tokio::fs::write(&stderr_path, &output.stderr).await?;
            let pdf_exists = pdf_path.exists();
            tracing::info!(
                target: "sidecar",
                exit_code = output.status.code(),
                pdf_exists,
                stdout_path = %stdout_path.display(),
                stderr_path = %stderr_path.display(),
                "latexmk completed"
            );
            Ok(result(
                output.status.success() && pdf_exists,
                engine,
                work_dir,
                main_tex_path,
                pdf_exists.then_some(pdf_path),
                log_path.exists().then_some(log_path),
                stdout_path,
                stderr_path,
                output.status.code(),
                (!output.status.success()).then(|| "latexmk exited with failure".to_string()),
            ))
        }
        Ok(Err(error)) => {
            write_text(&stdout_path, "").await?;
            write_text(&stderr_path, &error.to_string()).await?;
            Ok(result(
                false,
                engine,
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
            let message = "latexmk timed out".to_string();
            write_text(&stdout_path, "").await?;
            write_text(&stderr_path, &message).await?;
            Ok(result(
                false,
                engine,
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

async fn write_text(path: &PathBuf, value: &str) -> Result<()> {
    tokio::fs::write(path, value)
        .await
        .with_context(|| format!("writing {}", path.display()))
}

#[allow(clippy::too_many_arguments)]
fn result(
    ok: bool,
    engine: LatexEngine,
    work_dir: PathBuf,
    main_tex_path: PathBuf,
    pdf_path: Option<PathBuf>,
    log_path: Option<PathBuf>,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
    exit_code: Option<i32>,
    error_message: Option<String>,
) -> LatexCompileResult {
    LatexCompileResult {
        ok,
        engine,
        work_dir: work_dir.to_string_lossy().to_string(),
        main_tex_path: main_tex_path.to_string_lossy().to_string(),
        pdf_path: pdf_path.map(|path| path.to_string_lossy().to_string()),
        log_path: log_path.map(|path| path.to_string_lossy().to_string()),
        stdout_path: stdout_path.to_string_lossy().to_string(),
        stderr_path: stderr_path.to_string_lossy().to_string(),
        exit_code,
        error_message,
        finished_at: Utc::now().to_rfc3339(),
    }
}
