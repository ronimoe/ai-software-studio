//! core module — placeholder. Exists so commands and services can
//! depend on it as features land.

// TODO: implement in Phase core work.

pub mod worktree_context;
pub mod worktree_lifecycle;

#[cfg(test)]
mod worktree_context_tests;

#[cfg(test)]
mod worktree_lifecycle_tests;
