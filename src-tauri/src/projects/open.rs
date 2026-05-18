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
    let toplevel = resolve_git_toplevel(path_obj)?;

    let repo = ProjectRepository::new(db.clone());

    // Idempotency: if a project already exists for this canonical toplevel, return it.
    let existing = repo.list().await?;
    if let Some(found) = existing.into_iter().find(|p| p.path == toplevel) {
        return Ok(found);
    }

    let toplevel_path = Path::new(&toplevel);
    let name = toplevel_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("repo")
        .to_string();
    let default_branch = detect_default_branch(toplevel_path);

    let project = Project {
        id: format!("proj-{}", Uuid::new_v4()),
        name,
        path: toplevel,
        default_branch,
    };
    repo.insert(&project).await?;
    Ok(project)
}

fn resolve_git_toplevel(path: &Path) -> Result<String, AppError> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output()
        .map_err(|e| AppError::internal(format!("run git: {e}")))?;
    if !output.status.success() {
        return Err(AppError::invalid_arg(format!(
            "{} is not a git repository",
            path.display()
        )));
    }
    let toplevel = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if toplevel.is_empty() {
        return Err(AppError::invalid_arg(format!(
            "{} did not resolve to a git working tree",
            path.display()
        )));
    }
    Ok(toplevel)
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
