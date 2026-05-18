# AI Software Studio — User Manual

This is the end-user manual for AI Software Studio as it exists today
(**v0.0.2**, Plan 1 of v0.1 shipped). It describes what actually works right
now and what is still mocked. Read `docs/product-spec.md` and
`docs/architecture/architecture.md` for the broader vision.

---

## What you can do today (v0.0.2)

1. **Launch the app** as either a desktop window (Tauri) or a browser dev
   preview (Next.js + mock data).
2. **Open a git repository** as a project. The app validates the path is a
   git working tree, canonicalizes it to the repo root (so picking a
   subdirectory still opens the right project), and persists the entry in a
   local SQLite database.
3. **Switch between opened projects** via the dropdown in the header.
4. **See your projects survive restart** — the SQLite database lives at
   `~/Library/Application Support/AI Software Studio/app.db` on macOS, or
   the platform-equivalent data dir on Linux.

That is the entire interactive surface today. The dashboard panels (Task
Board, Engineering Snapshot, Agent Workspace, Review Room, Context Graph,
Conversation, Agent Manager) render but show mocked data. They become real
in Plans 2–7.

---

## Launching the app

### Desktop (Tauri)

```bash
pnpm tauri:dev
```

This compiles the Rust core, starts the Next.js dev server on port **1420**
(Tauri convention — chosen to avoid colliding with other Next.js projects
that default to 3000), and opens a native window. First launch takes ~1–2 minutes; subsequent
launches use the cargo cache and start in seconds.

You will need:

- **Node** ≥ 20
- **Rust** ≥ 1.88 (a `rust-toolchain.toml` pins this; rustup will
  auto-install if your default is older)
- **Xcode CLI tools** on macOS (`xcode-select --install`) or
  `build-essential` + `libwebkit2gtk-4.1-dev` etc. on Linux (see
  `.github/workflows/ci.yml` for the full Linux list)

### Browser mock mode (no Rust)

```bash
pnpm dev
```

Opens at `http://localhost:1420`. All Tauri commands are stubbed by
`lib/tauri.ts`'s mock dispatcher backed by `lib/mock-data.ts`. The "Open
project…" action falls back to a `window.prompt()` so the flow is still
usable without a native dialog.

This is the right mode for working on the UI without paying the Rust
compile cost.

---

## Opening your first project

1. Launch via `pnpm tauri:dev`.
2. The header shows "no workspace ▾" until you pick something.
3. Click the dropdown → "Open project…" → native directory picker opens.
4. Select any directory inside a git repository. The app runs
   `git rev-parse --show-toplevel` and stores the repo root, so:
   - Picking `~/Development/myrepo` → stored as `~/Development/myrepo`
   - Picking `~/Development/myrepo/src/components` → also stored as
     `~/Development/myrepo` (same project, no duplicate)
5. The dropdown updates with the repo's name and full path. That project
   is now active.
6. Re-opening the same path is idempotent — you get the same project back,
   not a new one.

Non-git directories are rejected with an error message containing "git".
Non-existent paths are rejected with a "does not exist" message. Both
errors show in the dev console; user-facing toasts are a Plan 4+ concern.

---

## Where your data lives

| Path | Contents |
|---|---|
| `~/Library/Application Support/AI Software Studio/app.db` (macOS) | SQLite database. Currently holds the `projects` table only. |
| `~/.local/share/AI Software Studio/app.db` (Linux) | Same, on Linux. |

To inspect:

```bash
sqlite3 "$HOME/Library/Application Support/AI Software Studio/app.db"
sqlite> .schema projects
sqlite> SELECT * FROM projects;
```

To reset (lose all opened-project history):

```bash
rm "$HOME/Library/Application Support/AI Software Studio/app.db"
```

The app will recreate it on next launch.

---

## Known limitations (v0.0.2)

- **The dashboard panels are mocked.** Task Board, Engineering Snapshot,
  Agent Workspace, Review Room, Context Graph, Conversation, and Agent
  Manager show fixture data from `lib/mock-data.ts`. They become real in
  Plans 2–7.
- **No real agent execution yet.** Claude Code and Codex CLI are not
  invoked; the engine adapter layer is stubbed.
- **No task creation, worktree management, or verification.** These land
  in Plans 3, 4, and 6.
- **No PR or GitHub integration.** That is Plan 7.
- **No migrations table** — the schema uses `CREATE TABLE IF NOT EXISTS`
  only. Plan 2 adds proper migration tracking.
- **`detect_default_branch` falls back to `"main"`** if both
  `git symbolic-ref refs/remotes/origin/HEAD` and `git branch --show-current`
  fail. Cosmetic for repos that use `master` or `trunk`.
- **No "open project cancelled" toast.** Cancelling the native picker is
  silent.
- **No way to remove or rename a persisted project from the UI.** Delete
  the row from SQLite manually for now (see "Where your data lives").
- **Windows is intentionally out of scope.** macOS and Linux only.

---

## Troubleshooting

### "It loaded another project's UI"

The Tauri webview points at `http://localhost:1420` (see
`src-tauri/tauri.conf.json` → `devUrl`). If another process is already on
that port, Tauri will render *that* process's UI.

Find and kill whatever's holding the port:

```bash
lsof -nP -i :1420
# Identify the PID, then:
kill -TERM <pid>
```

Then restart `pnpm tauri:dev`. **To change the port**, edit two places:

1. `src-tauri/tauri.conf.json` → `devUrl` (e.g. `"http://localhost:1430"`)
2. `package.json` → `scripts.dev` (e.g. `"next dev --port 1430"`)

Both must match. 1420 is Tauri's traditional default and is rarely held by
other tools; 3000 is the Next.js default and collides constantly.

### "Failed to export typescript bindings" / `out/` not found

`pnpm gen:bindings` runs `cargo test --lib export_bindings`. The lib build
gates `run()` behind `#[cfg(not(test))]` so the `tauri::generate_context!()`
proc-macro never expands during tests and the missing `out/` directory
isn't a problem. If you see this error, you probably ran `cargo check`
directly (not `cargo test --lib`) on the bin target — that's expected to
fail in dev because `out/` (Next.js export) only exists after `pnpm build`.

### Rust toolchain too old

The dep graph requires Rust ≥ 1.88 (via `time`, `darling`, `serde_with`,
`plist` transitives). The repo's `rust-toolchain.toml` pins this; rustup
should auto-install on `cargo` invocation. If it doesn't:

```bash
rustup install 1.88
rustup default 1.88
```

### Worktree node_modules conflict on `pnpm lint`

If you create a git worktree of this repo *inside* the repo (e.g. at
`.worktrees/feat-foo/`), eslint's plugin resolution may walk up and find
the parent `node_modules`, causing duplicate plugin loads and a
`circular structure to JSON` crash. Place worktrees *outside* the repo:

```bash
git worktree add ~/Development/ai-software-studio-worktrees/feat-foo feat/foo
```

---

## Roadmap (where this is going)

v0.1 plans, in order:

| Plan | Scope | Status |
|---|---|---|
| 0 | CI + Homebrew tap stub | Shipped (v0.0.1) |
| 1 | Persistence + project picker | Shipped (v0.0.2) ← **you are here** |
| 2 | Task model + intake wizard | Next |
| 3 | Git worktree + context capture | |
| 4 | Engine execution (Claude Code, Codex CLI) | |
| 5 | Changed files + diff viewer | |
| 6 | Verification (test/lint/typecheck/build) | |
| 7 | GitHub PR creation + evidence report | |

Each plan ships as its own `0.0.N` PATCH bump. The `0.1.0` MINOR is
reserved for the milestone where all 8 plans land.

---

## For developers

See `CLAUDE.md` (gitignored, local-only) for codebase conventions, the
Rust ↔ TS boundary contract, and how to add a new Tauri command.

See `docs/architecture/architecture.md` for the system-level design.

See `docs/superpowers/plans/2026-05-18-v0.1-plan-N-*.md` for the
implementation plans behind each shipped version.
