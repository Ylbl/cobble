use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    app_paths::AppPaths,
    gallery::{
        codex_groups,
        event_log::{self, append_error},
        groups::normalize_group_name,
        image_cache,
        persistence::{self, GallerySnapshot},
        types::{
            ArtifactItem, ArtifactKind, ArtifactSession, ArtifactStatus, ArtifactTurn, ClientName,
            CodexProjectGroup, SessionSource, SidebarMode,
        },
        view_model::{to_gallery_view, GalleryView},
    },
    latex::{
        compiler,
        types::{LatexCompileRequest, LatexCompileResult},
    },
    mcp::types::{
        ArtifactDisplayResult, ArtifactInput, ArtifactInputKind, ClientName as InputClientName,
        DisplayArtifactTurnInput, DisplayArtifactTurnResult, REUSE_INSTRUCTION,
    },
    settings::types::SidecarConfig,
};

pub struct GalleryState {
    app_paths: AppPaths,
    pub sessions: RwLock<Vec<ArtifactSession>>,
    pub selected_session_id: RwLock<Option<String>>,
    pub sidebar_mode: RwLock<SidebarMode>,
    pub codex_project_groups: RwLock<Vec<CodexProjectGroup>>,
}

impl GalleryState {
    pub async fn load(app_paths: AppPaths) -> Result<Self> {
        event_log::append_simple(&app_paths, "app_started", json!({})).await?;

        let loaded = match persistence::load(&app_paths).await {
            Ok(loaded) => loaded,
            Err(error) => {
                tracing::error!(target: "sidecar", ?error, "gallery-state.json load failed; using empty state");
                append_error(&app_paths, "gallery_load_failed", &error).await;
                persistence::LoadedGallerySnapshot {
                    snapshot: GallerySnapshot::default(),
                    existed: false,
                }
            }
        };

        event_log::append_simple(
            &app_paths,
            "gallery_loaded",
            json!({
                "existed": loaded.existed,
                "sessionCount": loaded.snapshot.sessions.len(),
            }),
        )
        .await?;

        let codex_project_groups = match codex_groups::load_project_groups(&app_paths).await {
            Ok(groups) => groups,
            Err(error) => {
                tracing::warn!(target: "sidecar", ?error, "Codex project groups unavailable");
                Vec::new()
            }
        };

        event_log::append_simple(
            &app_paths,
            "codex_project_groups_loaded",
            json!({ "count": codex_project_groups.len() }),
        )
        .await?;

        let sidebar_mode = if codex_project_groups.is_empty() {
            tracing::info!(
                target: "sidecar",
                "Codex project groups empty; falling back to sidebarMode=groups"
            );
            event_log::append_simple(
                &app_paths,
                "codex_project_groups_empty_fallback_to_groups",
                json!({ "reason": "codex project groups empty" }),
            )
            .await?;
            SidebarMode::Groups
        } else {
            loaded.snapshot.sidebar_mode
        };

        let state = Self {
            app_paths: app_paths.clone(),
            sessions: RwLock::new(loaded.snapshot.sessions),
            selected_session_id: RwLock::new(loaded.snapshot.selected_session_id),
            sidebar_mode: RwLock::new(sidebar_mode),
            codex_project_groups: RwLock::new(codex_project_groups),
        };

        if !loaded.existed {
            state.persist_current("gallery_state_saved").await?;
        }

        Ok(state)
    }

    pub fn app_paths(&self) -> &AppPaths {
        &self.app_paths
    }

    pub async fn snapshot(&self) -> GallerySnapshot {
        GallerySnapshot {
            sessions: self.sessions.read().await.clone(),
            selected_session_id: self.selected_session_id.read().await.clone(),
            sidebar_mode: self.sidebar_mode.read().await.clone(),
        }
    }

    pub async fn list_sessions(&self) -> Vec<ArtifactSession> {
        tracing::debug!(target: "sidecar", "front end requested gallery state");
        self.sessions.read().await.clone()
    }

    pub async fn list_view(&self) -> GalleryView {
        tracing::debug!(target: "sidecar", "front end requested gallery view");
        let sessions = self.sessions.read().await.clone();
        let selected_session_id = self.selected_session_id.read().await.clone();
        let sidebar_mode = self.sidebar_mode.read().await.clone();
        let codex_project_groups = self.codex_project_groups.read().await.clone();
        to_gallery_view(
            sessions,
            selected_session_id,
            sidebar_mode,
            codex_project_groups,
        )
    }

    pub async fn set_sidebar_mode(&self, mode: SidebarMode) -> Result<GalleryView> {
        *self.sidebar_mode.write().await = mode;
        self.persist_current("gallery_state_saved").await?;
        Ok(self.list_view().await)
    }

    pub async fn select_session(&self, session_id: String) -> Result<GalleryView> {
        let exists = self
            .sessions
            .read()
            .await
            .iter()
            .any(|session| session.id == session_id);
        if exists {
            *self.selected_session_id.write().await = Some(session_id);
            self.persist_current("gallery_state_saved").await?;
        }
        Ok(self.list_view().await)
    }

    pub async fn toggle_turn_collapsed(
        &self,
        session_id: String,
        turn_id: String,
    ) -> Result<GalleryView> {
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.iter_mut().find(|item| item.id == session_id) {
                if let Some(turn) = session.turns.iter_mut().find(|item| item.id == turn_id) {
                    turn.collapsed = !turn.collapsed;
                }
            }
        }
        self.persist_current("gallery_state_saved").await?;
        Ok(self.list_view().await)
    }

    pub async fn record_latex_smoke_test(
        &self,
        compile_result: LatexCompileResult,
    ) -> Result<GalleryView> {
        let now = now_string();
        let session_id = Uuid::new_v4().to_string();
        let turn_id = Uuid::new_v4().to_string();
        let artifact_id = Uuid::new_v4().to_string();
        let artifact = ArtifactItem {
            id: artifact_id.clone(),
            title: if compile_result.ok {
                "LaTeX Smoke Test".to_string()
            } else {
                "LaTeX Smoke Test Failed".to_string()
            },
            kind: if compile_result.ok {
                ArtifactKind::Pdf
            } else {
                ArtifactKind::Latex
            },
            status: if compile_result.ok {
                ArtifactStatus::Finished
            } else {
                ArtifactStatus::Failed
            },
            image_url: None,
            local_file_path: None,
            asset_url: None,
            pdf_url: None,
            pdf_local_file_path: compile_result.pdf_path.clone(),
            pdf_asset_url: None,
            log_file_path: compile_result.log_path.clone(),
            stdout_path: Some(compile_result.stdout_path.clone()),
            stderr_path: Some(compile_result.stderr_path.clone()),
            svg: None,
            latex_code: None,
            source_file_path: Some(compile_result.source_file_path.clone()),
            source_text: None,
            mime_type: Some("application/pdf".to_string()),
            file_extension: Some("pdf".to_string()),
            latex_engine: Some(compile_result.engine.clone()),
            compile_elapsed_ms: Some(compile_result.elapsed_ms),
            error_message: compile_result.error_message.clone(),
            created_at: now.clone(),
        };

        {
            let mut sessions = self.sessions.write().await;
            for session in &mut *sessions {
                for turn in &mut session.turns {
                    turn.collapsed = true;
                }
            }
            sessions.push(ArtifactSession {
                id: session_id.clone(),
                title: "LaTeX 环境检测".to_string(),
                source_kind: SessionSource::Manual,
                client_name: ClientName::Unknown,
                group_name: "默认分组".to_string(),
                project_name: String::new(),
                project_path: String::new(),
                created_at: now.clone(),
                updated_at: now.clone(),
                turns: vec![ArtifactTurn {
                    id: turn_id.clone(),
                    index: 1,
                    hint: Some("内置 LaTeX smoke test".to_string()),
                    created_at: now,
                    artifacts: vec![artifact],
                    collapsed: false,
                }],
            });
        }
        *self.selected_session_id.write().await = Some(session_id.clone());
        self.append_event(
            "latex_smoke_test_artifact_created",
            json!({
                "sidecarSessionId": session_id,
                "sidecarTurnId": turn_id,
                "artifactId": artifact_id,
                "ok": compile_result.ok,
            }),
        )
        .await;
        self.persist_current("gallery_state_saved").await?;
        Ok(self.list_view().await)
    }

    pub async fn display_artifact_turn(
        &self,
        input: DisplayArtifactTurnInput,
        config: SidecarConfig,
    ) -> DisplayArtifactTurnResult {
        tracing::info!(target: "sidecar", artifact_count = input.artifacts.len(), "display_artifact_turn received");

        let prepared_artifacts = self.prepare_artifacts(&input.artifacts, &config).await;

        let mut created_new_session = false;
        let mut session_created_event = None;
        let now = now_string();
        let turn_id = Uuid::new_v4().to_string();
        let mut artifact_ids = Vec::new();
        let sidecar_session_id;
        {
            let mut sessions = self.sessions.write().await;
            let session_index = input
                .sidecar_session_id
                .as_ref()
                .and_then(|id| sessions.iter().position(|session| &session.id == id));

            let session_index = match session_index {
                Some(index) => {
                    tracing::debug!(target: "sidecar", sidecar_session_id = %sessions[index].id, "reusing session");
                    index
                }
                None => {
                    created_new_session = true;
                    let id = Uuid::new_v4().to_string();
                    let title = normalize_text(input.session_title.as_deref(), "未命名会话");
                    tracing::info!(target: "sidecar", sidecar_session_id = %id, "creating session");
                    sessions.push(ArtifactSession {
                        id: id.clone(),
                        title,
                        source_kind: SessionSource::Mcp,
                        client_name: input_client_to_domain(input.client_name.clone()),
                        group_name: normalize_group_name(input.group_name.as_deref().unwrap_or("")),
                        project_name: normalize_optional(input.project_name.as_deref()),
                        project_path: normalize_optional(input.project_path.as_deref()),
                        created_at: now.clone(),
                        updated_at: now.clone(),
                        turns: Vec::new(),
                    });
                    session_created_event = Some(id);
                    sessions.len() - 1
                }
            };

            let session = &mut sessions[session_index];
            update_session_from_input(session, &input);

            for turn in &mut session.turns {
                turn.collapsed = true;
            }

            let turn_index = session.turns.len() as u32 + 1;
            for artifact in &prepared_artifacts {
                artifact_ids.push(artifact.id.clone());
            }

            tracing::info!(
                target: "sidecar",
                sidecar_session_id = %session.id,
                sidecar_turn_id = %turn_id,
                turn_index,
                artifact_count = prepared_artifacts.len(),
                "creating turn"
            );

            session.updated_at = now.clone();
            session.turns.push(ArtifactTurn {
                id: turn_id.clone(),
                index: turn_index,
                hint: input.turn_hint.clone(),
                created_at: now.clone(),
                artifacts: prepared_artifacts,
                collapsed: false,
            });
            sidecar_session_id = session.id.clone();
        }

        *self.selected_session_id.write().await = Some(sidecar_session_id.clone());

        if let Some(created_session_id) = session_created_event {
            self.append_event(
                "session_created",
                json!({ "sidecarSessionId": created_session_id }),
            )
            .await;
        }
        self.append_event(
            "turn_created",
            json!({
                "sidecarSessionId": sidecar_session_id,
                "sidecarTurnId": turn_id,
                "artifactCount": artifact_ids.len(),
            }),
        )
        .await;
        for artifact_id in &artifact_ids {
            self.append_event(
                "artifact_created",
                json!({
                    "sidecarSessionId": sidecar_session_id,
                    "sidecarTurnId": turn_id,
                    "artifactId": artifact_id,
                }),
            )
            .await;
        }
        self.append_lifecycle_events(&sidecar_session_id, &turn_id)
            .await;

        if let Err(error) = self.persist_current("gallery_state_saved").await {
            tracing::error!(target: "sidecar", ?error, "failed to persist gallery state");
        }

        let displayed = !artifact_ids.is_empty();
        tracing::info!(
            target: "sidecar",
            sidecar_session_id = %sidecar_session_id,
            sidecar_turn_id = %turn_id,
            artifact_count = artifact_ids.len(),
            "display_artifact_turn completed"
        );

        let artifact_results = self.artifact_results(&artifact_ids).await;

        DisplayArtifactTurnResult {
            ok: true,
            sidecar_session_id,
            sidecar_turn_id: turn_id,
            artifact_ids,
            artifact_results,
            created_new_session,
            displayed,
            message: if displayed {
                "Artifacts displayed in Sidecar.".to_string()
            } else {
                "No supported artifacts were provided.".to_string()
            },
            reuse_instruction: REUSE_INSTRUCTION.to_string(),
        }
    }

    async fn prepare_artifacts(
        &self,
        artifacts: &[ArtifactInput],
        config: &SidecarConfig,
    ) -> Vec<ArtifactItem> {
        let mut prepared = Vec::new();
        let now = now_string();

        for artifact in artifacts {
            match artifact.kind {
                ArtifactInputKind::Image => {
                    let artifact_id = Uuid::new_v4().to_string();
                    let item = self
                        .prepare_image_artifact(artifact, artifact_id, &now)
                        .await;
                    if let Err(error) =
                        image_cache::write_artifact_manifest(&self.app_paths, &item.id, &item).await
                    {
                        tracing::error!(target: "sidecar", ?error, artifact_id = %item.id, "failed to write artifact manifest");
                    }
                    prepared.push(item);
                }
                ArtifactInputKind::Latex => {
                    let artifact_id = format!("art_{}", Uuid::new_v4());
                    let item = self
                        .prepare_latex_artifact(artifact, artifact_id, &now, config)
                        .await;
                    if let Err(error) =
                        image_cache::write_artifact_manifest(&self.app_paths, &item.id, &item).await
                    {
                        tracing::error!(target: "sidecar", ?error, artifact_id = %item.id, "failed to write artifact manifest");
                    }
                    prepared.push(item);
                }
                _ => {
                    tracing::warn!(target: "sidecar", kind = ?artifact.kind, "unsupported artifact kind ignored");
                }
            }
        }

        prepared
    }

    async fn prepare_image_artifact(
        &self,
        artifact: &ArtifactInput,
        artifact_id: String,
        now: &str,
    ) -> ArtifactItem {
        let Some(image_url) = artifact.image_url.clone() else {
            let error_message = "image artifact requires imageUrl".to_string();
            tracing::warn!(target: "sidecar", artifact_id = %artifact_id, "image artifact missing imageUrl");
            return ArtifactItem {
                id: artifact_id,
                title: artifact.title.clone(),
                kind: ArtifactKind::Image,
                status: ArtifactStatus::Failed,
                image_url: None,
                local_file_path: None,
                asset_url: None,
                pdf_url: None,
                pdf_local_file_path: None,
                pdf_asset_url: None,
                log_file_path: None,
                stdout_path: None,
                stderr_path: None,
                svg: None,
                latex_code: None,
                source_file_path: None,
                source_text: None,
                mime_type: None,
                file_extension: None,
                latex_engine: None,
                compile_elapsed_ms: None,
                error_message: Some(error_message),
                created_at: now.to_string(),
            };
        };

        self.append_event(
            "image_download_started",
            json!({ "artifactId": artifact_id, "imageUrl": image_url }),
        )
        .await;
        let download =
            image_cache::cache_remote_image(&self.app_paths, &artifact_id, &image_url).await;
        if download.ok {
            self.append_event(
                "image_download_finished",
                json!({
                    "artifactId": artifact_id,
                    "statusCode": download.status_code,
                    "contentType": download.content_type,
                    "localFilePath": download.local_file_path,
                }),
            )
            .await;
        } else {
            self.append_event(
                "image_download_failed",
                json!({
                    "artifactId": artifact_id,
                    "errorMessage": download.error_message,
                }),
            )
            .await;
        }

        tracing::info!(
            target: "sidecar",
            artifact_id = %artifact_id,
            status = if download.ok { "finished" } else { "failed" },
            "artifact status changed"
        );

        ArtifactItem {
            id: artifact_id,
            title: artifact.title.clone(),
            kind: ArtifactKind::Image,
            status: if download.ok {
                ArtifactStatus::Finished
            } else {
                ArtifactStatus::Failed
            },
            image_url: Some(image_url),
            local_file_path: download.local_file_path.clone(),
            asset_url: None,
            pdf_url: None,
            pdf_local_file_path: None,
            pdf_asset_url: None,
            log_file_path: None,
            stdout_path: None,
            stderr_path: None,
            svg: None,
            latex_code: None,
            source_file_path: None,
            source_text: None,
            mime_type: download.content_type.clone(),
            file_extension: download.file_extension.clone(),
            latex_engine: None,
            compile_elapsed_ms: None,
            error_message: download.error_message.clone(),
            created_at: now.to_string(),
        }
    }

    async fn prepare_latex_artifact(
        &self,
        artifact: &ArtifactInput,
        artifact_id: String,
        now: &str,
        config: &SidecarConfig,
    ) -> ArtifactItem {
        let Some(latex_code) = artifact.latex_code.clone().filter(|value| !value.trim().is_empty())
        else {
            let error_message = "latex artifact requires latexCode".to_string();
            tracing::warn!(target: "sidecar", artifact_id = %artifact_id, "latex artifact missing latexCode");
            return ArtifactItem {
                id: artifact_id,
                title: artifact.title.clone(),
                kind: ArtifactKind::Latex,
                status: ArtifactStatus::Failed,
                image_url: None,
                local_file_path: None,
                asset_url: None,
                pdf_url: None,
                pdf_local_file_path: None,
                pdf_asset_url: None,
                log_file_path: None,
                stdout_path: None,
                stderr_path: None,
                svg: None,
                latex_code: None,
                source_file_path: None,
                source_text: None,
                mime_type: None,
                file_extension: None,
                latex_engine: artifact.latex_engine.clone(),
                compile_elapsed_ms: None,
                error_message: Some(error_message),
                created_at: now.to_string(),
            };
        };

        let engine = artifact
            .latex_engine
            .clone()
            .unwrap_or_else(|| config.latex.engine.clone());
        let work_dir = self
            .app_paths
            .artifacts_dir
            .join(sanitize_filename::sanitize(&artifact_id));

        tracing::info!(
            target: "sidecar",
            artifact_id = %artifact_id,
            latex_chars = latex_code.chars().count(),
            engine = ?engine,
            work_dir = %work_dir.display(),
            "MCP latex artifact received"
        );
        self.append_event(
            "latex_artifact_received",
            json!({
                "artifactId": artifact_id,
                "title": artifact.title,
                "latexChars": latex_code.chars().count(),
                "engine": engine,
                "workDir": work_dir,
            }),
        )
        .await;
        self.append_event(
            "latex_compile_started",
            json!({
                "artifactId": artifact_id,
                "engine": engine,
                "workDir": work_dir,
                "timeoutSeconds": config.latex.compile_timeout_seconds,
            }),
        )
        .await;

        let compile = compiler::compile_latex_artifact(LatexCompileRequest {
            artifact_id: artifact_id.clone(),
            title: artifact.title.clone(),
            latex_code: latex_code.clone(),
            engine: engine.clone(),
            work_dir: work_dir.to_string_lossy().to_string(),
            timeout_seconds: config.latex.compile_timeout_seconds,
        })
        .await;

        match compile {
            Ok(result) => {
                let status = if result.ok {
                    ArtifactStatus::Finished
                } else {
                    ArtifactStatus::Failed
                };
                self.append_event(
                    "latex_source_written",
                    json!({
                        "artifactId": artifact_id,
                        "sourceFilePath": result.source_file_path,
                    }),
                )
                .await;
                self.append_event(
                    if result.ok {
                        "latex_compile_finished"
                    } else {
                        "latex_compile_failed"
                    },
                    json!({
                        "artifactId": artifact_id,
                        "engine": result.engine,
                        "elapsedMs": result.elapsed_ms,
                        "exitCode": result.exit_code,
                        "pdfFilePath": result.pdf_file_path,
                        "logFilePath": result.log_file_path,
                        "stdoutPath": result.stdout_path,
                        "stderrPath": result.stderr_path,
                        "errorMessage": result.error_message,
                    }),
                )
                .await;
                self.append_event(
                    "artifact_log_saved",
                    json!({
                        "artifactId": artifact_id,
                        "logFilePath": result.log_file_path,
                        "stdoutPath": result.stdout_path,
                        "stderrPath": result.stderr_path,
                    }),
                )
                .await;
                if result.ok {
                    self.append_event(
                        "pdf_artifact_ready",
                        json!({
                            "artifactId": artifact_id,
                            "pdfFilePath": result.pdf_file_path,
                        }),
                    )
                    .await;
                }
                tracing::info!(
                    target: "sidecar",
                    artifact_id = %artifact_id,
                    status = ?status,
                    pdf_exists = result.pdf_file_path.is_some(),
                    elapsed_ms = result.elapsed_ms,
                    "latex artifact status updated"
                );
                ArtifactItem {
                    id: artifact_id,
                    title: artifact.title.clone(),
                    kind: ArtifactKind::Latex,
                    status,
                    image_url: None,
                    local_file_path: None,
                    asset_url: None,
                    pdf_url: None,
                    pdf_local_file_path: result.pdf_file_path.clone(),
                    pdf_asset_url: None,
                    log_file_path: result.log_file_path.clone(),
                    stdout_path: Some(result.stdout_path.clone()),
                    stderr_path: Some(result.stderr_path.clone()),
                    svg: None,
                    latex_code: Some(latex_code),
                    source_file_path: Some(result.source_file_path.clone()),
                    source_text: None,
                    mime_type: result.ok.then(|| "application/pdf".to_string()),
                    file_extension: result.ok.then(|| "pdf".to_string()),
                    latex_engine: Some(result.engine.clone()),
                    compile_elapsed_ms: Some(result.elapsed_ms),
                    error_message: result.error_message.clone(),
                    created_at: now.to_string(),
                }
            }
            Err(error) => {
                let message = error.to_string();
                self.append_event(
                    "latex_compile_failed",
                    json!({
                        "artifactId": artifact_id,
                        "errorMessage": message,
                    }),
                )
                .await;
                ArtifactItem {
                    id: artifact_id,
                    title: artifact.title.clone(),
                    kind: ArtifactKind::Latex,
                    status: ArtifactStatus::Failed,
                    image_url: None,
                    local_file_path: None,
                    asset_url: None,
                    pdf_url: None,
                    pdf_local_file_path: None,
                    pdf_asset_url: None,
                    log_file_path: None,
                    stdout_path: None,
                    stderr_path: None,
                    svg: None,
                    latex_code: Some(latex_code),
                    source_file_path: None,
                    source_text: None,
                    mime_type: None,
                    file_extension: None,
                    latex_engine: Some(engine),
                    compile_elapsed_ms: None,
                    error_message: Some(message),
                    created_at: now.to_string(),
                }
            }
        }
    }

    async fn append_lifecycle_events(&self, session_id: &str, turn_id: &str) {
        let payloads = {
            let sessions = self.sessions.read().await;
            let Some(session) = sessions.iter().find(|item| item.id == session_id) else {
                return;
            };
            let Some(turn) = session.turns.iter().find(|item| item.id == turn_id) else {
                return;
            };
            turn.artifacts
                .iter()
                .filter(|artifact| matches!(artifact.kind, ArtifactKind::Latex))
                .map(|artifact| {
                    json!({
                        "sessionId": session_id,
                        "turnId": turn_id,
                        "artifactId": artifact.id,
                        "status": artifact.status,
                        "summary": artifact.error_message.clone().unwrap_or_else(|| "latex artifact processed".to_string()),
                        "paths": {
                            "sourceFilePath": artifact.source_file_path,
                            "pdfLocalFilePath": artifact.pdf_local_file_path,
                            "logFilePath": artifact.log_file_path,
                            "stdoutPath": artifact.stdout_path,
                            "stderrPath": artifact.stderr_path,
                        }
                    })
                })
                .collect::<Vec<_>>()
        };

        for payload in payloads {
            self.append_event("latex_artifact_lifecycle", payload).await;
        }
    }

    async fn artifact_results(&self, artifact_ids: &[String]) -> Vec<ArtifactDisplayResult> {
        let sessions = self.sessions.read().await;
        sessions
            .iter()
            .flat_map(|session| &session.turns)
            .flat_map(|turn| &turn.artifacts)
            .filter(|artifact| artifact_ids.contains(&artifact.id))
            .map(|artifact| ArtifactDisplayResult {
                artifact_id: artifact.id.clone(),
                kind: artifact_kind_label(&artifact.kind).to_string(),
                status: artifact_status_label(&artifact.status).to_string(),
                pdf_generated: artifact.pdf_local_file_path.is_some()
                    && matches!(artifact.status, ArtifactStatus::Finished),
                message: artifact_result_message(artifact),
            })
            .collect()
    }

    async fn persist_current(&self, event_type: &str) -> Result<()> {
        let snapshot = self.snapshot().await;
        persistence::save(&self.app_paths, &snapshot).await?;
        self.append_event(
            event_type,
            json!({
                "sessionCount": snapshot.sessions.len(),
            }),
        )
        .await;
        Ok(())
    }

    async fn append_event(&self, event_type: &str, payload: serde_json::Value) {
        if let Err(error) = event_log::append_simple(&self.app_paths, event_type, payload).await {
            tracing::error!(target: "sidecar", ?error, event_type, "failed to append gallery event");
        }
    }
}

fn update_session_from_input(session: &mut ArtifactSession, input: &DisplayArtifactTurnInput) {
    if let Some(session_title) = input
        .session_title
        .as_ref()
        .map(|title| title.trim())
        .filter(|title| !title.is_empty())
    {
        if session.title != session_title {
            tracing::info!(
                target: "sidecar",
                sidecar_session_id = %session.id,
                old_title = %session.title,
                new_title = %session_title,
                "updating session title"
            );
            session.title = session_title.to_string();
        }
    }
    if let Some(group_name) = input.group_name.as_deref() {
        session.group_name = normalize_group_name(group_name);
    }
    if let Some(client_name) = input.client_name.clone() {
        session.client_name = input_client_to_domain(Some(client_name));
    }
    if let Some(project_name) = input.project_name.as_deref() {
        session.project_name = normalize_optional(Some(project_name));
    }
    if let Some(project_path) = input.project_path.as_deref() {
        session.project_path = normalize_optional(Some(project_path));
    }
}

fn input_client_to_domain(client_name: Option<InputClientName>) -> ClientName {
    match client_name.unwrap_or(InputClientName::Unknown) {
        InputClientName::Codex => ClientName::Codex,
        InputClientName::ZCode => ClientName::ZCode,
        InputClientName::Cursor => ClientName::Cursor,
        InputClientName::Unknown => ClientName::Unknown,
    }
}

fn artifact_kind_label(kind: &ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::Image => "image",
        ArtifactKind::Pdf => "pdf",
        ArtifactKind::Latex => "latex",
        ArtifactKind::Svg => "svg",
    }
}

fn artifact_status_label(status: &ArtifactStatus) -> &'static str {
    match status {
        ArtifactStatus::Received => "received",
        ArtifactStatus::Rendering => "rendering",
        ArtifactStatus::Compiling => "compiling",
        ArtifactStatus::Finished => "finished",
        ArtifactStatus::Failed => "failed",
    }
}

fn artifact_result_message(artifact: &ArtifactItem) -> String {
    match (&artifact.kind, &artifact.status) {
        (ArtifactKind::Latex, ArtifactStatus::Finished) => {
            "LaTeX compiled successfully and PDF is displayed in Sidecar.".to_string()
        }
        (ArtifactKind::Latex, ArtifactStatus::Failed) => {
            "LaTeX compilation failed. Check source and log in Sidecar.".to_string()
        }
        (ArtifactKind::Image, ArtifactStatus::Finished) => {
            "Image downloaded and displayed in Sidecar.".to_string()
        }
        (_, ArtifactStatus::Failed) => artifact
            .error_message
            .clone()
            .unwrap_or_else(|| "Artifact processing failed.".to_string()),
        _ => "Artifact processed in Sidecar.".to_string(),
    }
}

fn normalize_text(value: Option<&str>, fallback: &str) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn normalize_optional(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("")
        .to_string()
}

fn now_string() -> String {
    Utc::now().to_rfc3339()
}
