use std::collections::BTreeMap;

use crate::gallery::{
    types::{ArtifactSession, CodexProjectGroup, DEFAULT_GROUP_NAME},
    view_model::{GalleryGroupView, GalleryProjectView},
};

pub fn build_group_views(sessions: &[ArtifactSession]) -> Vec<GalleryGroupView> {
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    groups.insert(DEFAULT_GROUP_NAME.to_string(), Vec::new());

    for session in sessions {
        groups
            .entry(normalize_group_name(&session.group_name))
            .or_default()
            .push(session.id.clone());
    }

    // Sort session_ids within each group by session updated_at desc
    let session_order: std::collections::HashMap<&str, usize> = sessions
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id.as_str(), i))
        .collect();

    groups
        .into_iter()
        .map(|(name, mut session_ids)| {
            session_ids.sort_by_key(|id| session_order.get(id.as_str()).copied().unwrap_or(usize::MAX));
            GalleryGroupView {
                id: format!("group:{}", name),
                name,
                session_count: session_ids.len(),
                session_ids,
            }
        })
        .collect()
}

pub fn build_project_views(
    sessions: &[ArtifactSession],
    codex_groups: &[CodexProjectGroup],
) -> Vec<GalleryProjectView> {
    let session_order: std::collections::HashMap<&str, usize> = sessions
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id.as_str(), i))
        .collect();

    let mut projects: BTreeMap<String, GalleryProjectView> = BTreeMap::new();

    for project in codex_groups {
        projects.insert(
            project.path.clone(),
            GalleryProjectView {
                id: project.id.clone(),
                name: project.name.clone(),
                path: project.path.clone(),
                session_ids: Vec::new(),
                session_count: 0,
            },
        );
    }

    for session in sessions {
        let project_path = session.project_path.trim();
        if project_path.is_empty() {
            continue;
        }
        let entry =
            projects
                .entry(project_path.to_string())
                .or_insert_with(|| GalleryProjectView {
                    id: format!("project:{}", project_path),
                    name: if session.project_name.trim().is_empty() {
                        project_path.to_string()
                    } else {
                        session.project_name.clone()
                    },
                    path: project_path.to_string(),
                    session_ids: Vec::new(),
                    session_count: 0,
                });
        entry.session_ids.push(session.id.clone());
        entry.session_count = entry.session_ids.len();
    }

    // Sort session_ids within each project by session updated_at desc
    projects
        .into_values()
        .map(|mut p| {
            p.session_ids.sort_by_key(|id| session_order.get(id.as_str()).copied().unwrap_or(usize::MAX));
            p
        })
        .collect()
}

pub fn normalize_group_name(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        DEFAULT_GROUP_NAME.to_string()
    } else {
        trimmed.to_string()
    }
}
