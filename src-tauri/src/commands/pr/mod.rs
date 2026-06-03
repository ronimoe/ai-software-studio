pub mod report;

#[cfg(test)]
mod report_tests;

use crate::{
    error::AppError,
    models::{CreatePrRequest, GitHubStatus, PrResult},
    state::AppState,
};
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn detect_github() -> Result<GitHubStatus, AppError> {
    tokio::task::spawn_blocking(crate::engines::github::detect)
        .await
        .map_err(|e| AppError::internal(format!("join: {e}")))?
}

#[tauri::command]
#[specta::specta]
pub async fn render_pr_report(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<String, AppError> {
    let task = state.tasks.get(&task_id).await?;
    let runs = state.verification.list_for_task(&task_id).await?;
    let latest = runs.first().cloned();

    let worktree = task.worktree_path.clone();
    let changed = if let Some(wt) = worktree {
        let p = PathBuf::from(wt);
        tokio::task::spawn_blocking(move || crate::git::status::status(&p))
            .await
            .map_err(|e| AppError::internal(format!("join: {e}")))??
    } else {
        Vec::new()
    };

    Ok(report::render(&task, &changed, latest.as_ref()))
}

#[tauri::command]
#[specta::specta]
pub async fn create_pr(
    state: State<'_, AppState>,
    request: CreatePrRequest,
) -> Result<PrResult, AppError> {
    let task = state.tasks.get(&request.task_id).await?;
    let project = state.projects.get(&task.project_id).await?;
    let branch = task
        .branch_name
        .clone()
        .ok_or_else(|| AppError::invalid_arg("task has no branch; create a worktree first"))?;
    let worktree = task
        .worktree_path
        .clone()
        .ok_or_else(|| AppError::invalid_arg("task has no worktree"))?;
    let base = request
        .base_branch
        .clone()
        .unwrap_or(project.default_branch.clone());

    let repo_path = PathBuf::from(&worktree);
    let branch_clone = branch.clone();
    tokio::task::spawn_blocking(move || {
        crate::engines::github::push_branch(&repo_path, &branch_clone)
    })
    .await
    .map_err(|e| AppError::internal(format!("join: {e}")))??;

    let runs = state.verification.list_for_task(&task.id).await?;
    let latest = runs.first().cloned();
    let changed = {
        let p = PathBuf::from(&worktree);
        tokio::task::spawn_blocking(move || crate::git::status::status(&p))
            .await
            .map_err(|e| AppError::internal(format!("join: {e}")))??
    };
    let body = report::render(&task, &changed, latest.as_ref());

    let repo_path = PathBuf::from(&worktree);
    let title = task.title.clone();
    let base_clone = base.clone();
    let url = tokio::task::spawn_blocking(move || {
        crate::engines::github::create_pr(&repo_path, &title, &body, &base_clone, request.draft)
    })
    .await
    .map_err(|e| AppError::internal(format!("join: {e}")))??;

    state
        .tasks
        .update_status(&task.id, crate::models::TaskStatus::PrPrepared)
        .await?;

    Ok(PrResult { url, branch, base })
}
