# AI Software Studio

**AI Software Studio is a local-first desktop application that lets developers assign coding tasks to local AI agents such as Claude Code and Codex CLI, monitor their work, verify results, review evidence, and approve changes safely.**

## Overview

AI Software Studio turns local AI coding agents into a structured engineering workflow. It provides the missing workflow layer for local AI coding agents, moving from an unstructured chat/terminal experience to a controlled, evidence-backed process.

The key shift:
- **Old workflow:** Human asks AI to edit code → AI modifies files → human manually inspects everything.
- **AI Software Studio:** Human defines task and constraints → agent works in isolated worktree → app captures output, diff, tests, and risks → human reviews evidence → human approves or rejects.

## Documentation

Full documentation is available in the `docs` folder:

- [Documentation Index](./docs/README.md)
- [Product Brief](./docs/product-brief.md)
- [Product Spec](./docs/product-spec.md)
- [Architecture Details](./docs/architecture/README.md)
- [Exploration Notes](./docs/exploration/README.md)

## Core Principles

- **Local-first**: Core functionality runs locally without a hosted backend.
- **No provider token storage**: Uses developer's existing tools (Claude Code/Codex CLI) which handle their own authentication.
- **Human-approved**: AI agents execute, but humans own the final decision.
- **Evidence-backed**: Verifies agent output independently through Git diffs, test runs, logs, and reports.
- **Isolated by default**: Agent work happens in a dedicated Git worktree.
- **Engine-agnostic**: Designed to support multiple local engines.

## Running Locally

**Prerequisites:** Node.js ≥ 20, pnpm ≥ 9, Rust ≥ 1.78 (install via `rustup`), Xcode Command Line Tools (macOS) or `build-essential` (Linux).

```bash
pnpm install
pnpm gen:bindings   # generates lib/bindings.ts from Rust types
pnpm dev            # browser-only iteration via mock dispatcher
pnpm tauri:dev      # full desktop window with Rust backend
```

Other scripts:

- `pnpm test` — unit tests (vitest)
- `pnpm typecheck` — TypeScript strict check
- `pnpm lint` — ESLint
- `pnpm build` — static export to `out/`
- `pnpm tauri:build` — desktop bundle (`.app` on macOS, `.AppImage`/`.deb` on Linux)

`lib/bindings.ts` is generated from Rust and `.gitignore`d. If your IDE complains it doesn't exist, run `pnpm gen:bindings`.
