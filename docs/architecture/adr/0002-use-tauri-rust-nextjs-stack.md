# ADR-002: Use Tauri + Rust + Next.js Stack

## Status

Accepted

## Context

AI Software Studio needs a rich desktop UI and strong local system access.

The app must:

- render a modern task and review dashboard
- run local commands
- control CLI agents
- manage Git worktrees
- stream terminal output
- store local artifacts
- support macOS and Linux

## Decision

Use:

```text
Next.js + React + TypeScript for UI
Tauri for desktop shell
Rust for native local runtime
SQLite for local persistence
```

## Consequences

### Positive

- Next.js enables fast UI development
- Tauri provides lightweight desktop packaging
- Rust provides strong local execution control
- Better long-term security posture than putting local command control in frontend code
- Good fit for macOS and Linux
- Clear separation between UI and privileged native operations

### Negative

- Requires Rust knowledge
- More complex than a pure Next.js local web app
- Tauri + Next.js packaging needs testing
- PTY and process management require careful implementation

## Alternatives Considered

### Electron + Next.js + Node

Better for rapid MVP, but heavier and less attractive as a final architecture.

### Local Next.js Web App Only

Simpler, but less polished and weaker for desktop integration.

### Vite + Tauri

Simpler than Next.js for desktop UI, but Next.js is preferred for app structure and developer familiarity.

## Revisit When

- Tauri packaging blocks progress
- Rust process control becomes too costly
- Next.js static export becomes incompatible with required UI behavior
