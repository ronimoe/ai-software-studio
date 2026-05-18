# ADR-005: Use Git Worktrees for Task Isolation

## Status

Accepted

## Context

AI coding agents may modify many files. Running them directly in the user's active working tree is risky.

The app needs isolation so each task can be reviewed, reverted, and compared cleanly.

## Decision

Use **Git worktrees** for task isolation.

Each task gets:

- its own branch
- its own worktree directory
- isolated agent execution
- independent diff and verification

Example branch:

```text
aistudio/task-12-magic-link-login
```

Example worktree:

```text
~/.local/share/ai-software-studio/worktrees/example-app/task-12
```

## Consequences

### Positive

- Protects the main working tree
- Makes changed files and diffs easier to inspect
- Enables multiple tasks later
- Easier cleanup and rollback
- Clear branch-to-task mapping

### Negative

- Requires Git knowledge and careful state management
- Worktree creation can fail in unusual repo states
- Storage usage increases
- Submodules and monorepos may require extra handling

## Alternatives Considered

### Direct Editing in Current Working Tree

Rejected because it is too risky.

### Temporary Copy of Repository

Rejected because it is slower, larger, and harder to keep connected to Git history.

### Docker Workspace

Possible later, but too heavy for initial local-first workflow.

## Revisit When

- Git worktrees cause reliability issues
- containerized execution becomes necessary
- multi-repo workflows become common
