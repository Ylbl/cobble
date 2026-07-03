use std::{fs, path::PathBuf};

use anyhow::Context;
use chrono::Local;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    app_paths::AppPaths,
    gallery::{
        persistence::GallerySnapshot,
        types::{ArtifactItem, ArtifactKind, ArtifactSession, ArtifactTurn},
        view_model::GalleryView,
    },
    mcp::types::{ArtifactInputKind, DisplayArtifactTurnInput, DisplayArtifactTurnResult},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResolvedSessionDebug<'a> {
    sidecar_session_id: &'a str,
    session_title: &'a str,
    created_new_session: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreatedArtifactsDebug<'a> {
    artifact_ids: &'a [String],
    artifacts: Vec<&'a ArtifactItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageDownloadPlanDebug<'a> {
    artifact_index: usize,
    title: &'a str,
    image_url: Option<&'a str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageDownloadResultDebug<'a> {
    artifact_id: &'a str,
    title: &'a str,
    status: &'a crate::gallery::types::ArtifactStatus,
    image_url: Option<&'a str>,
    local_file_path: Option<&'a str>,
    mime_type: Option<&'a str>,
    file_extension: Option<&'a str>,
    error_message: Option<&'a str>,
}

pub fn write_run(
    app_paths: &AppPaths,
    input: &DisplayArtifactTurnInput,
    result: &DisplayArtifactTurnResult,
    snapshot_before: &GallerySnapshot,
    snapshot_after: &GallerySnapshot,
    view_after: &GalleryView,
) -> anyhow::Result<PathBuf> {
    let short_id = Uuid::new_v4()
        .to_string()
        .split('-')
        .next()
        .unwrap_or("run")
        .to_string();
    let run_dir = app_paths.debug_artifacts_dir.join(format!(
        "run-{}-{short_id}",
        Local::now().format("%Y%m%d-%H%M%S")
    ));
    fs::create_dir_all(&run_dir).with_context(|| format!("creating {}", run_dir.display()))?;

    let session = snapshot_after
        .sessions
        .iter()
        .find(|session| session.id == result.sidecar_session_id);
    let turn = session.and_then(|session| {
        session
            .turns
            .iter()
            .find(|turn| turn.id == result.sidecar_turn_id)
    });

    write_json(run_dir.join("mcp-request.json"), input)?;
    if let Some(latex_input) = input
        .artifacts
        .iter()
        .find(|artifact| matches!(artifact.kind, ArtifactInputKind::Latex))
    {
        write_json(run_dir.join("latex-artifact-input.json"), latex_input)?;
        if let Some(code) = latex_input.latex_code.as_deref() {
            fs::write(run_dir.join("latex-source-main.tex"), code)
                .with_context(|| format!("writing {}", run_dir.join("latex-source-main.tex").display()))?;
        }
    }
    write_json(run_dir.join("image-download-plan.json"), &image_plan(input))?;
    if let Some(turn) = turn {
        write_json(
            run_dir.join("image-download-result.json"),
            &image_results(turn, result),
        )?;
        write_latex_evidence(&run_dir, turn, result)?;
    }
    write_json(run_dir.join("persistence-before.json"), snapshot_before)?;
    write_json(run_dir.join("persistence-after.json"), snapshot_after)?;
    copy_event_log(app_paths, &run_dir)?;
    write_json(run_dir.join("gallery-view-after.json"), view_after)?;
    if let Some(session) = session {
        write_json(
            run_dir.join("resolved-session.json"),
            &ResolvedSessionDebug {
                sidecar_session_id: &session.id,
                session_title: &session.title,
                created_new_session: result.created_new_session,
            },
        )?;
    }
    if let Some(turn) = turn {
        write_json(run_dir.join("created-turn.json"), turn)?;
        write_created_artifacts(&run_dir, turn, result)?;
    }
    write_json(run_dir.join("tool-result.json"), result)?;
    write_summary(&run_dir, session, turn, result)?;

    tracing::info!(target: "sidecar", path = %run_dir.display(), "debug artifact run written");
    Ok(run_dir)
}

fn image_plan(input: &DisplayArtifactTurnInput) -> Vec<ImageDownloadPlanDebug<'_>> {
    input
        .artifacts
        .iter()
        .enumerate()
        .map(|(index, artifact)| ImageDownloadPlanDebug {
            artifact_index: index,
            title: &artifact.title,
            image_url: artifact.image_url.as_deref(),
        })
        .collect()
}

fn image_results<'a>(
    turn: &'a ArtifactTurn,
    result: &DisplayArtifactTurnResult,
) -> Vec<ImageDownloadResultDebug<'a>> {
    turn.artifacts
        .iter()
        .filter(|artifact| result.artifact_ids.contains(&artifact.id))
        .map(|artifact| ImageDownloadResultDebug {
            artifact_id: &artifact.id,
            title: &artifact.title,
            status: &artifact.status,
            image_url: artifact.image_url.as_deref(),
            local_file_path: artifact.local_file_path.as_deref(),
            mime_type: artifact.mime_type.as_deref(),
            file_extension: artifact.file_extension.as_deref(),
            error_message: artifact.error_message.as_deref(),
        })
        .collect()
}

fn copy_event_log(app_paths: &AppPaths, run_dir: &std::path::Path) -> anyhow::Result<()> {
    let output_path = run_dir.join("gallery-event.jsonl");
    if app_paths.gallery_events_path.exists() {
        fs::copy(&app_paths.gallery_events_path, &output_path).with_context(|| {
            format!(
                "copying {} to {}",
                app_paths.gallery_events_path.display(),
                output_path.display()
            )
        })?;
    } else {
        fs::write(&output_path, "")?;
    }
    Ok(())
}

fn write_latex_evidence(
    run_dir: &std::path::Path,
    turn: &ArtifactTurn,
    result: &DisplayArtifactTurnResult,
) -> anyhow::Result<()> {
    let Some(artifact) = turn.artifacts.iter().find(|artifact| {
        result.artifact_ids.contains(&artifact.id) && matches!(artifact.kind, ArtifactKind::Latex)
    }) else {
        return Ok(());
    };

    write_json(
        run_dir.join("latex-compile-command.json"),
        &serde_json::json!({
            "artifactId": artifact.id,
            "engine": artifact.latex_engine,
            "workDir": artifact.source_file_path.as_deref().and_then(|path| std::path::Path::new(path).parent()).map(|path| path.display().to_string()),
            "command": [
                "latexmk",
                artifact.latex_engine.as_ref().map(|engine| engine.latexmk_arg()).unwrap_or(""),
                "-interaction=nonstopmode",
                "-halt-on-error",
                "-file-line-error",
                "-outdir=out",
                "main.tex"
            ],
        }),
    )?;
    write_json(
        run_dir.join("latex-compile-result.json"),
        &serde_json::json!({
            "artifactId": artifact.id,
            "status": artifact.status,
            "pdfLocalFilePath": artifact.pdf_local_file_path,
            "sourceFilePath": artifact.source_file_path,
            "logFilePath": artifact.log_file_path,
            "stdoutPath": artifact.stdout_path,
            "stderrPath": artifact.stderr_path,
            "compileElapsedMs": artifact.compile_elapsed_ms,
            "errorMessage": artifact.error_message,
        }),
    )?;

    copy_optional_file(run_dir, "latex-source-main.tex", artifact.source_file_path.as_deref())?;
    copy_optional_file(run_dir, "latex-stdout.txt", artifact.stdout_path.as_deref())?;
    copy_optional_file(run_dir, "latex-stderr.txt", artifact.stderr_path.as_deref())?;
    copy_optional_file(run_dir, "latex-main.log", artifact.log_file_path.as_deref())?;
    Ok(())
}

fn copy_optional_file(
    run_dir: &std::path::Path,
    output_name: &str,
    source: Option<&str>,
) -> anyhow::Result<()> {
    let Some(source) = source else {
        return Ok(());
    };
    let source_path = std::path::Path::new(source);
    if source_path.exists() {
        fs::copy(source_path, run_dir.join(output_name)).with_context(|| {
            format!(
                "copying {} to {}",
                source_path.display(),
                run_dir.join(output_name).display()
            )
        })?;
    }
    Ok(())
}

fn write_created_artifacts(
    run_dir: &std::path::Path,
    turn: &ArtifactTurn,
    result: &DisplayArtifactTurnResult,
) -> anyhow::Result<()> {
    let artifacts = turn
        .artifacts
        .iter()
        .filter(|artifact| result.artifact_ids.contains(&artifact.id))
        .collect();
    write_json(
        run_dir.join("created-artifacts.json"),
        &CreatedArtifactsDebug {
            artifact_ids: &result.artifact_ids,
            artifacts,
        },
    )
}

fn write_json(path: PathBuf, value: &(impl Serialize + ?Sized)) -> anyhow::Result<()> {
    fs::write(&path, serde_json::to_string_pretty(value)?)
        .with_context(|| format!("writing {}", path.display()))
}

fn write_summary(
    run_dir: &std::path::Path,
    session: Option<&ArtifactSession>,
    turn: Option<&ArtifactTurn>,
    result: &DisplayArtifactTurnResult,
) -> anyhow::Result<()> {
    let summary = format!(
        "# display_artifact_turn\n\n- ok: {}\n- displayed: {}\n- sidecarSessionId: {}\n- sidecarTurnId: {}\n- artifactIds: {}\n- session: {}\n- turn: {}\n\n{}\n",
        result.ok,
        result.displayed,
        result.sidecar_session_id,
        result.sidecar_turn_id,
        result.artifact_ids.join(", "),
        session.map(|item| item.title.as_str()).unwrap_or("unknown"),
        turn.map(|item| item.index.to_string()).unwrap_or_else(|| "unknown".to_string()),
        result.reuse_instruction,
    );
    fs::write(run_dir.join("summary.md"), summary)
        .with_context(|| format!("writing {}", run_dir.join("summary.md").display()))
}
