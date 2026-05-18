use crate::{db::Db, error::AppError, models::Project, projects::repository::ProjectRepository};
use std::path::Path;
use std::process::Command;
use uuid::Uuid;

pub async fn open_project(db: &Db, path: &str) -> Result<Project, AppError> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(AppError::invalid_arg(format!("path does not exist: {path}")));
    }
    if !path_obj.is_dir() {
        return Err(AppError::invalid_arg(format!("path is not a directory: {path}")));
    }
    validate_is_git_repo(path_obj)?;

    let repo = ProjectRepository::new(db.clone());

    // Idempotency: if a project already exists for this exact path, return it.
    let existing = repo.list().await?;
    if let Some(found) = existing.into_iter().find(|p| p.path == path) {
        return Ok(found);
    }

    let name = path_obj
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("repo")
        .to_string();
    let default_branch = detect_default_branch(path_obj);

    let project = Project {
        id: format!("proj-{}", Uuid::new_v4()),
        name,
        path: path.to_string(),
        default_branch,
    };
    repo.insert(&project).await?;
    Ok(project)
}

fn validate_is_git_repo(path: &Path) -> Result<(), AppError> {
    let output = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(path)
        .output()
        .map_err(|e| AppError::internal(format!("run git: {e}")))?;
    if !output.status.success() {
        return Err(AppError::invalid_arg(format!(
            "{} is not a git repository",
            path.display()
        )));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim() != "true" {
        return Err(AppError::invalid_arg(format!(
            "{} is not the top of a git working tree",
            path.display()
        )));
    }
    Ok(())
}

fn detect_default_branch(path: &Path) -> String {
    // Try `git symbolic-ref refs/remotes/origin/HEAD`. Fall back to current branch.
    if let Ok(out) = Command::new("git")
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .current_dir(path)
        .output()
    {
        if out.status.success() {
            if let Some(name) = String::from_utf8_lossy(&out.stdout)
                .trim()
                .strip_prefix("refs/remotes/origin/")
            {
                return name.to_string();
            }
        }
    }
    if let Ok(out) = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(path)
        .output()
    {
        if out.status.success() {
            let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !name.is_empty() {
                return name;
            }
        }
    }
    "main".to_string()
}
