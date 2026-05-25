use crate::{error::AppError, models::GitHubStatus};
use std::ffi::OsStr;
use std::path::Path;

pub fn detect() -> Result<GitHubStatus, AppError> {
    Err(AppError::unimplemented("gh detect"))
}

pub fn push_branch(_repo: &Path, _branch: &str) -> Result<(), AppError> {
    Err(AppError::unimplemented("push"))
}

pub fn create_pr(
    _repo: &Path,
    _title: &str,
    _body: &str,
    _base: &str,
    _draft: bool,
) -> Result<String, AppError> {
    Err(AppError::unimplemented("create_pr"))
}

// Pure helper: extract the GitHub login from `gh auth status` output.
// Looks for lines like "Logged in to github.com as ronimoe (keyring)".
pub(super) fn parse_account(_s: &str) -> Option<String> {
    None
}

// PATH-injectable executable lookup. Returns the absolute path of an
// executable named `name` found in `path_var`, or None if not found.
pub(super) fn which_in(_name: &str, _path_var: &OsStr) -> Option<String> {
    None
}
