use crate::{error::AppError, models::TaskStatus, tasks::TaskService};
use std::path::PathBuf;

/// On startup, reconcile tasks left mid-flight by a previous (crashed/quit)
/// session. Running/VerificationRunning → ReviewReady (changes) or Stopped
/// (clean). Never auto-retries; Queued tasks are untouched (the worker drains them).
pub async fn reconcile_orphans(tasks: &TaskService) -> Result<(), AppError> {
    let ids = tasks
        .ids_in_statuses(&["running", "verificationRunning"])
        .await?;
    for id in ids {
        let task = tasks.get(&id).await?;
        let new_status = match &task.worktree_path {
            Some(wt) => {
                let p = PathBuf::from(wt);
                let changed = tokio::task::spawn_blocking(move || crate::git::status::status(&p))
                    .await
                    .map_err(|e| AppError::internal(format!("join: {e}")))?
                    .unwrap_or_default();
                if changed.is_empty() {
                    TaskStatus::Stopped
                } else {
                    TaskStatus::ReviewReady
                }
            }
            None => TaskStatus::Stopped,
        };
        tasks.update_status(&id, new_status).await?;
    }
    Ok(())
}
