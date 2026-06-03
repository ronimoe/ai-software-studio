use crate::error::AppError;
use std::path::{Path, PathBuf};

pub fn worktree_root() -> Result<PathBuf, AppError> {
    let base = dirs::data_dir()
        .ok_or_else(|| AppError::internal("no platform data dir"))?
        .join("AI Software Studio")
        .join("worktrees");
    std::fs::create_dir_all(&base)
        .map_err(|e| AppError::internal(format!("create worktrees root: {e}")))?;
    Ok(base)
}

pub fn worktree_path(project_id: &str, task_id: &str) -> Result<PathBuf, AppError> {
    Ok(worktree_root()?.join(project_id).join(task_id))
}

pub fn branch_name(task_id: &str) -> String {
    let short = task_id.strip_prefix("task-").unwrap_or(task_id);
    let safe = short.chars().take(8).collect::<String>();
    format!("aistudio/task-{safe}")
}

/// Refuse-list guard for any operation that turns a stored `worktree_path`
/// back into a filesystem effect (delete, scan, etc.).
///
/// A corrupted or imported SQLite row could point `worktree_path` anywhere on
/// disk; without this check, `remove_worktree` would happily nuke
/// `/Users/me`. When the path exists, both sides are canonicalised so symlinks
/// and `..` tricks can't escape the managed root. When the path no longer
/// exists (legitimate post-cleanup state), fall back to a string-prefix check
/// against the absolute `worktree_root()` — best we can do without an inode.
pub fn is_within_worktree_root(path: &Path) -> bool {
    let Ok(root) = worktree_root() else {
        return false;
    };
    if path.exists() {
        let Ok(canon_path) = path.canonicalize() else {
            return false;
        };
        let Ok(canon_root) = root.canonicalize() else {
            return false;
        };
        canon_path.starts_with(&canon_root)
    } else {
        // worktree_root() returns an absolute path, so a string-prefix check
        // here is safe — there's no relative-path ambiguity to exploit.
        path.starts_with(&root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_name_truncates_uuid_for_readability() {
        let n = branch_name("task-12345678-aaaa-bbbb-cccc-dddddddddddd");
        assert_eq!(n, "aistudio/task-12345678");
    }

    #[test]
    fn branch_name_handles_short_id() {
        let n = branch_name("task-042");
        assert_eq!(n, "aistudio/task-042");
    }

    #[test]
    fn worktree_path_segments_match_layout() {
        let p = worktree_path("proj-1", "task-1").expect("ok");
        let s = p.to_string_lossy();
        assert!(s.contains("AI Software Studio/worktrees/proj-1/task-1"));
    }

    // --- is_within_worktree_root ---------------------------------------------
    //
    // These tests operate against the real `worktree_root()` (the platform
    // data dir). That's intentional: `is_within_worktree_root` is a guard
    // around _that_ root specifically, and mocking it would test the mock
    // rather than the contract. Tests create paths under the real root and
    // clean them up afterwards.

    use std::fs;

    fn unique_suffix() -> String {
        format!("guard-test-{}", uuid::Uuid::new_v4())
    }

    #[test]
    fn is_within_root_accepts_existing_path_inside_root() {
        let root = worktree_root().expect("root");
        let inside = root.join(unique_suffix());
        fs::create_dir_all(&inside).expect("mkdir inside");
        let result = is_within_worktree_root(&inside);
        let _ = fs::remove_dir_all(&inside);
        assert!(result, "path under managed root must be allowed");
    }

    #[test]
    fn is_within_root_accepts_root_itself() {
        let root = worktree_root().expect("root");
        // Defining: a path equal to the root counts as "within" — both
        // canonicalised forms are equal, and `starts_with` is reflexive.
        assert!(is_within_worktree_root(&root));
    }

    #[test]
    fn is_within_root_rejects_path_outside_root() {
        // /tmp is reliably outside the platform data dir on macOS and Linux.
        let outside = std::env::temp_dir();
        assert!(
            !is_within_worktree_root(&outside),
            "path outside managed root must be rejected"
        );
    }

    #[test]
    fn is_within_root_accepts_nonexistent_path_with_correct_prefix() {
        let root = worktree_root().expect("root");
        let phantom = root.join(unique_suffix()).join("never-created");
        assert!(!phantom.exists(), "precondition: path does not exist");
        assert!(
            is_within_worktree_root(&phantom),
            "nonexistent path with the right prefix is legitimately post-cleanup"
        );
    }

    #[test]
    fn is_within_root_rejects_nonexistent_path_with_wrong_prefix() {
        let phantom = std::env::temp_dir()
            .join(unique_suffix())
            .join("never-created");
        assert!(!phantom.exists(), "precondition: path does not exist");
        assert!(
            !is_within_worktree_root(&phantom),
            "nonexistent path outside the root must still be rejected"
        );
    }

    #[cfg(unix)]
    #[test]
    fn is_within_root_rejects_symlink_pointing_outside() {
        use std::os::unix::fs::symlink;
        let root = worktree_root().expect("root");
        let link_path = root.join(unique_suffix());
        let target = std::env::temp_dir();
        symlink(&target, &link_path).expect("create symlink");
        let result = is_within_worktree_root(&link_path);
        let _ = fs::remove_file(&link_path);
        assert!(
            !result,
            "a symlink whose target is outside the root must be rejected after canonicalisation"
        );
    }
}
