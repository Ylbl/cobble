use std::{fs, path::PathBuf};

use anyhow::Context;
use chrono::Local;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    gallery::types::{ArtifactSession, ArtifactTurn},
    mcp::types::{DisplayArtifactTurnInput, DisplayArtifactTurnResult},
    paths,
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
    artifacts: Vec<&'a crate::gallery::types::ArtifactItem>,
}

pub fn write_run(
    input: &DisplayArtifactTurnInput,
    result: &DisplayArtifactTurnResult,
    sessions_after: &[ArtifactSession],
) -> anyhow::Result<PathBuf> {
    let short_id = Uuid::new_v4()
        .to_string()
        .split('-')
        .next()
        .unwrap_or("run")
        .to_string();
    let run_dir = paths::debug_artifacts_dir().join(format!(
        "run-{}-{short_id}",
        Local::now().format("%Y%m%d-%H%M%S")
    ));
    fs::create_dir_all(&run_dir).with_context(|| format!("creating {}", run_dir.display()))?;

    let session = sessions_after
        .iter()
        .find(|session| session.id == result.sidecar_session_id);
    let turn = session.and_then(|session| {
        session
            .turns
            .iter()
            .find(|turn| turn.id == result.sidecar_turn_id)
    });

    write_json(run_dir.join("mcp-request.json"), input)?;
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
    write_json(run_dir.join("gallery-state-after.json"), sessions_after)?;
    write_json(run_dir.join("tool-result.json"), result)?;
    write_summary(&run_dir, session, turn, result)?;

    tracing::info!(target: "sidecar", path = %run_dir.display(), "debug artifact run written");
    Ok(run_dir)
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
