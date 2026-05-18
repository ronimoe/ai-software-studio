# Design Spec: AI Software Studio — App Scaffold & Next.js Template

**Date:** 2026-05-17
**Status:** Draft for review
**Topic:** Create initial monorepo scaffold (Tauri v2 + Rust + Next.js) and build the Next.js UI to match the visual template in `ui.png`.

---

## 1. Goal

Stand up the initial application skeleton so that:

1. `pnpm dev` boots the Next.js UI in a browser, rendering the full dashboard from the template with mock data.
2. `pnpm tauri dev` opens the same UI inside a Tauri v2 desktop window, with Rust command stubs returning the same mock data through the IPC bridge.
3. All architectural seams from `docs/architecture/architecture.md` are in place — Zustand stores, TanStack Query, typed Tauri boundary, feature modules — even though most domain logic (Git, engine detection, SQLite, PTY, verification) is deferred to later phases.

Outcome: a working visual prototype that already wears the correct architecture, ready for incremental backend implementation.

---

## 2. Scope

### In scope

- Full monorepo scaffold: Tauri v2 (`src-tauri/`) + Next.js 15 App Router (root) + pnpm.
- Tailwind v4 + shadcn/ui + dark/light theme with toggle (dark default).
- All 7 dashboard panels from `ui.png` rendered with mock data.
- Typed Tauri command bridge (`lib/tauri.ts`) generated via `tauri-specta`, with dev-mode fallback to in-memory mocks.
- Rust command stubs for tasks, projects, engines, verification — returning fixtures through service-layer indirection (Architecture §9).
- **Stubbed engine detection** (Architecture §16 Phase 1 minimum): `detect_engines` command returns hardcoded `EngineStatus` results identifying Claude Code and Codex CLI by name. Real `which`/`--version` shelling deferred. Treat this scaffold as Phase 0 + Phase 1 minimum so the panel inventory has a real binding to validate against.
- Zustand stores for UI state, TanStack Query for server state.
- Conditional `git init` (only if no `.git` exists at repo root) + appropriate `.gitignore`.

### Out of scope (explicitly deferred)

- Real engine `which`/`--version` detection (stubbed detection IS in scope; see above)
- Real Git/worktree operations
- SQLite/sqlx wiring
- PTY/process runner for agent execution
- Verification command execution
- Policy engine, sensitive-path detection
- Routing — single-page dashboard is sufficient
- High-fidelity network graph in Context Graph (placeholder SVG acceptable)
- Code signing / notarization for `tauri build`

These get module placeholders or are left absent. They have their own phases in `docs/architecture/architecture.md` §16.

---

## 3. Repository Structure

Flat Tauri v2 layout (Next.js at root, `src-tauri/` alongside) — standard for `create-tauri-app`, easiest tooling.

```
ai-software-studio/
├── app/
│   ├── layout.tsx
│   ├── page.tsx
│   ├── globals.css
│   └── providers.tsx
├── components/
│   ├── ui/                     # shadcn primitives
│   ├── layout/                 # DashboardShell, PanelFrame, AppHeader, ThemeToggle
│   └── panels/
│       ├── task-board/
│       ├── agent-workspace/
│       ├── review-room/
│       ├── context-graph/      # contains active-agents.tsx sub-component
│       ├── conversation/
│       ├── agent-manager/
│       └── engineering-snapshot/
├── features/
│   ├── projects/
│   ├── tasks/
│   ├── engines/
│   ├── worktrees/
│   ├── verification/
│   └── artifacts/
├── lib/
│   ├── tauri.ts
│   ├── mock-data.ts
│   ├── types.ts
│   └── utils.ts
├── stores/
│   ├── task-store.ts
│   ├── engine-store.ts
│   └── ui-store.ts
├── public/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── state.rs              # AppState managed by Tauri (holds service handles)
│   │   ├── error.rs              # AppError { code, message, details? } — serializable
│   │   ├── models.rs             # Serde + specta types (camelCase wire format)
│   │   ├── fixtures.rs           # mock data mirroring lib/mock-data.ts
│   │   ├── commands/             # Tauri commands — thin layer, delegate to services
│   │   │   ├── mod.rs
│   │   │   ├── projects.rs
│   │   │   ├── tasks.rs
│   │   │   ├── engines.rs
│   │   │   └── verification.rs
│   │   ├── core/                 # placeholder mod.rs (TODO: domain types)
│   │   ├── engines/              # placeholder mod.rs (TODO: EngineAdapter trait)
│   │   ├── git/                  # placeholder mod.rs (TODO: worktree service)
│   │   ├── process/              # placeholder mod.rs (TODO: PTY/process runner)
│   │   ├── verification/         # placeholder mod.rs (TODO: verification runner)
│   │   ├── policy/               # placeholder mod.rs (TODO: sensitive-path policy)
│   │   ├── artifacts/            # placeholder mod.rs (TODO: artifact store)
│   │   ├── db/                   # placeholder mod.rs (TODO: sqlx pool)
│   │   └── config/               # placeholder mod.rs (TODO: .aistudio/config.yaml)
│   ├── tauri.conf.json
│   ├── Cargo.toml
│   ├── build.rs
│   ├── icons/
│   └── capabilities/
│       └── default.json
├── docs/                        # existing
├── package.json
├── pnpm-lock.yaml
├── next.config.ts
├── tsconfig.json
├── postcss.config.mjs
├── components.json              # shadcn config
├── .gitignore
└── README.md                    # existing, append "Running locally" section
```

Engineering Snapshot is its own panel folder (smaller widget pinned to the bottom of the left column). Engine Status does NOT get its own panel — engine cards live inside `context-graph/active-agents.tsx`.

---

## 4. Frontend Architecture

### 4.1 Page composition

Single route `app/page.tsx` renders `<DashboardShell>`. No routing — the product fits one window.

3-column CSS Grid:

```
grid-cols-[280px_1fr_360px]
grid-rows-[auto_1fr]   (header row + content row)
```

Layout:

```
┌──────────────────────────── AppHeader ─────────────────────────┐
│ Brand · Title + tagline · Workspace switcher · ThemeToggle · Avatar │
├──────────┬────────────────────────────┬────────────────────────┤
│ Task     │ Agent Workspace            │ Context Graph          │
│ Board    │                            │  (Active Agents +      │
│          │                            │   node graph)          │
│          ├────────────────────────────┼────────────────────────┤
│          │ Review Room                │ Conversation           │
│ Eng.     │  (status pills +           ├────────────────────────┤
│ Snapshot │   evidence artifacts)      │ Agent Manager          │
└──────────┴────────────────────────────┴────────────────────────┘
```

Desktop-only proportions; min window 1200×800.

### 4.2 Panel module shape

Each `components/panels/<name>/`:
- `index.tsx` — panel component wrapping `<PanelFrame>` with body
- `<name>.types.ts` — local types (if not shared via `features/`)
- co-located subcomponents (e.g., `task-card.tsx`, `active-agents.tsx`)

`<PanelFrame>` props: `title`, `badge?`, `actions?`, `className?`, `children`. Provides the glass surface, header row, padding.

### 4.3 State management

| Concern | Mechanism |
|---|---|
| Server data (tasks, engines, verification) | TanStack Query keyed by IDs, fetched via feature hooks |
| Cross-panel UI cursor (active task, modal open) | Zustand `ui-store` |
| Streaming agent output (append-heavy) | Zustand `task-store.streamingLog` |
| Theme | `next-themes` (own state) |

`ui-store` holds: `activeTaskId`, `activeProjectId`, `agentManagerOpen`. Server data flows through `useTasks()`, `useTask(id)`, `useEngines()`, `useVerification(taskId)` in `features/*`.

### 4.4 Tauri boundary

The boundary uses **`tauri-specta`** so command signatures and Serde-derived types are exported as TypeScript at build time. No hand-maintained `Commands` map.

Rust side (sketch):

```rust
// commands/tasks.rs
#[tauri::command]
#[specta::specta]
pub async fn list_tasks(
    state: tauri::State<'_, AppState>,
    project_id: String,
) -> Result<Vec<Task>, AppError> {
    state.tasks.list_for_project(&project_id).await
}
```

`build.rs` (or a `lib.rs` test) invokes `tauri_specta::Builder` to emit `lib/bindings.ts` containing:
- Typed wrappers for each command (e.g., `export async function listTasks(projectId: string): Promise<Task[]>`)
- All `models.rs` types as TS interfaces (Task, Project, EngineStatus, VerificationRun, AppError)

`lib/tauri.ts` re-exports those wrappers and adds:
- A `safeCall(...)` helper that funnels `AppError` rejections into a normalized shape for TanStack Query
- The dev-mode mock dispatcher (see §4.5)

Components never touch `@tauri-apps/api` directly. They call feature hooks; hooks call the generated wrappers from `lib/bindings.ts` (or the `safeCall` indirection).

**Wire format.** Rust types use `#[serde(rename_all = "camelCase")]` so JS receives `projectId`, not `project_id`. `tauri-specta` emits matching TS. Command function names stay snake_case in Rust per Tauri convention; specta generates camelCase TS wrappers automatically.

**Error type.** `AppError` is defined in `src-tauri/src/error.rs`:

```rust
#[derive(Debug, Serialize, specta::Type, thiserror::Error)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: AppErrorCode,        // enum: NotFound, InvalidArg, Internal, EngineNotReady, ...
    pub message: String,
    pub details: Option<serde_json::Value>,
}
```

Replaces `Result<T, String>` everywhere.

### 4.5 Dev mode (no Tauri)

`lib/tauri.ts` detects the Tauri runtime using the official `isTauri()` from `@tauri-apps/api/core` (not the internal `window.__TAURI_INTERNALS__`). If `isTauri()` returns false, the generated wrappers are monkey-patched at module load to route through an in-memory dispatcher reading from `lib/mock-data.ts`. Same shapes as Rust fixtures. Lets us iterate the UI in `pnpm dev` (browser) without rebuilding Rust.

---

## 5. Tauri / Rust Skeleton

### 5.1 Module layout

Full layout in §3 above. Key points:

- `state.rs` defines `AppState` — managed via `tauri::Builder::manage(...)` — holding `Arc` handles to each service. Today services are mock-backed; tomorrow they read from SQLite, Git, processes.
- `error.rs` defines `AppError` (see §4.4).
- `commands/` contains thin command wrappers — they read `tauri::State<AppState>`, delegate to a service method, return the service's result. No business logic in command modules.
- `core/`, `db/`, `git/`, `process/`, `engines/`, `verification/`, `policy/`, `artifacts/`, `config/` ship as placeholder modules with a `mod.rs` containing only a doc comment + `// TODO: implement in Phase N` marker. This locks in the architecture from day one so later phases drop code into existing seams instead of creating new ones.

### 5.2 Service layer (mock-backed initially)

Each service module exposes an async-friendly API. For the scaffold pass we provide just enough surface to back the commands listed below — full traits get fleshed out as features land.

```rust
// engines/mod.rs (scaffold form)
use crate::{error::AppError, models::EngineStatus};

pub struct EngineService;

impl EngineService {
    pub fn new() -> Self { Self }

    pub async fn list(&self) -> Result<Vec<EngineStatus>, AppError> {
        Ok(crate::fixtures::engines())
    }

    pub async fn detect(&self) -> Result<Vec<EngineStatus>, AppError> {
        // Phase 1 stub: returns the same fixture as `list`.
        // Replaced with real `which` / `--version` shelling in later phase.
        Ok(crate::fixtures::engines())
    }
}
```

`AppState` holds one instance per service:

```rust
// state.rs
pub struct AppState {
    pub tasks: tasks::TaskService,
    pub projects: projects::ProjectService,
    pub engines: engines::EngineService,
    pub verification: verification::VerificationService,
}
```

### 5.3 Command stub example

```rust
// commands/tasks.rs
use crate::{error::AppError, models::Task, state::AppState};

#[tauri::command]
#[specta::specta]
pub async fn list_tasks(
    state: tauri::State<'_, AppState>,
    project_id: String,
) -> Result<Vec<Task>, AppError> {
    state.tasks.list_for_project(&project_id).await
}
```

Async, structured error, body delegates to a service method. Function bodies stay this thin forever — real logic moves into the service, not the command.

### 5.4 Tauri v2 config (`tauri.conf.json`)

- `productName`: `"AI Software Studio"`
- `identifier`: `"studio.aisoftware.app"`
- `app.windows[0]`: title `"AI Software Studio"`, width 1440, height 900, minWidth 1200, minHeight 800, resizable true
- `build.frontendDist`: `"../out"`
- `build.devUrl`: `"http://localhost:3000"`
- `build.beforeDevCommand`: `"pnpm dev"`
- `build.beforeBuildCommand`: `"pnpm build"`

Capabilities (`src-tauri/capabilities/default.json`): minimal v2 set, `core:default` only. No filesystem, shell, or HTTP plugins requested in this scaffold — added in later phases as commands need them.

### 5.5 Next.js config for static export

`next.config.ts`:

```ts
const config: NextConfig = {
  output: 'export',
  images: { unoptimized: true },
  trailingSlash: true,
};
```

App Router stays. Use **only static-export-compatible** features. The following are **forbidden** in this codebase and will break `next build`:

- Route Handlers (`app/**/route.ts`)
- Server Actions (`'use server'`)
- Middleware (`middleware.ts`)
- ISR / `revalidate` exports
- Dynamic route segments without `generateStaticParams`
- `next/image` optimizer (we set `images.unoptimized: true`)
- Request-time APIs: `cookies()`, `headers()`, `draftMode()`
- Node-only modules (`fs`, `child_process`, etc.) in client code

Server Components themselves are fine as long as they don't use any of the above — but in practice every panel needs interactivity, so the dashboard is client-component-heavy. The rule is "no runtime server work," not "everything must be `'use client'`."

Add an ESLint rule (`next/no-server-import-in-page` plus a custom check for `'use server'`) in a follow-up phase.

---

## 6. Visual Theme

### 6.1 Aesthetic

From `ui.png`: deep navy/charcoal canvas, glass panel cards (thin border, subtle gradient), indigo/purple primary, cyan secondary, lime/green for "passed", soft pinks for warnings. Rounded corners (~`rounded-xl`).

### 6.2 Token approach

Tailwind v4 CSS-first, **two-layer pattern** matching shadcn's documented v4 setup:

1. **Semantic CSS variables** (`--background`, `--foreground`, `--primary`, etc.) defined under `:root` (light values) and `.dark` (dark values that override). Shadcn's generated component CSS reads these via `var(--background)`. next-themes flips between modes by toggling the `.dark` class on `<html>` (see §6.3); even though the default theme is dark, `:root` still holds the light palette per shadcn convention.
2. **`@theme inline { ... }`** maps those semantic vars to Tailwind utility colors (`--color-background: var(--background)`) so utilities like `bg-background` resolve correctly.

No `tailwind.config.ts` is needed.

```css
/* app/globals.css */
@import "tailwindcss";

/* Light is the inverted-default per shadcn convention */
:root {
  --background: oklch(0.98 0.005 250);
  --foreground: oklch(0.20 0.02 265);
  --card: oklch(1 0 0);
  --card-foreground: oklch(0.20 0.02 265);
  --popover: oklch(1 0 0);
  --popover-foreground: oklch(0.20 0.02 265);
  --primary: oklch(0.55 0.20 290);
  --primary-foreground: oklch(0.98 0 0);
  --secondary: oklch(0.62 0.14 200);
  --secondary-foreground: oklch(0.10 0.02 265);
  --muted: oklch(0.96 0.005 250);
  --muted-foreground: oklch(0.50 0.02 260);
  --accent: oklch(0.65 0.16 330);
  --accent-foreground: oklch(0.10 0.02 265);
  --destructive: oklch(0.55 0.22 25);
  --destructive-foreground: oklch(0.98 0 0);
  --success: oklch(0.62 0.18 145);
  --warning: oklch(0.70 0.16 75);
  --border: oklch(0.90 0.01 250);
  --input: oklch(0.92 0.01 250);
  --ring: oklch(0.55 0.20 290);
  --radius: 0.875rem;
}

/* Dark is the canonical look matching ui.png */
.dark {
  --background: oklch(0.18 0.025 265);
  --foreground: oklch(0.95 0.01 250);
  --card: oklch(0.22 0.03 265);
  --card-foreground: oklch(0.95 0.01 250);
  --popover: oklch(0.22 0.03 265);
  --popover-foreground: oklch(0.95 0.01 250);
  --primary: oklch(0.65 0.20 290);
  --primary-foreground: oklch(0.98 0 0);
  --secondary: oklch(0.78 0.14 200);
  --secondary-foreground: oklch(0.10 0.02 265);
  --muted: oklch(0.26 0.025 265);
  --muted-foreground: oklch(0.70 0.02 260);
  --accent: oklch(0.75 0.16 330);
  --accent-foreground: oklch(0.10 0.02 265);
  --destructive: oklch(0.65 0.22 25);
  --destructive-foreground: oklch(0.98 0 0);
  --success: oklch(0.78 0.18 145);
  --warning: oklch(0.80 0.16 75);
  --border: oklch(0.32 0.03 265 / 0.6);
  --input: oklch(0.32 0.03 265);
  --ring: oklch(0.65 0.20 290);
}

@theme inline {
  --color-background: var(--background);
  --color-foreground: var(--foreground);
  --color-card: var(--card);
  --color-card-foreground: var(--card-foreground);
  --color-popover: var(--popover);
  --color-popover-foreground: var(--popover-foreground);
  --color-primary: var(--primary);
  --color-primary-foreground: var(--primary-foreground);
  --color-secondary: var(--secondary);
  --color-secondary-foreground: var(--secondary-foreground);
  --color-muted: var(--muted);
  --color-muted-foreground: var(--muted-foreground);
  --color-accent: var(--accent);
  --color-accent-foreground: var(--accent-foreground);
  --color-destructive: var(--destructive);
  --color-destructive-foreground: var(--destructive-foreground);
  --color-success: var(--success);
  --color-warning: var(--warning);
  --color-border: var(--border);
  --color-input: var(--input);
  --color-ring: var(--ring);
  --radius-lg: var(--radius);
  --font-sans: "Inter", ui-sans-serif, system-ui, sans-serif;
  --font-mono: "JetBrains Mono", ui-monospace, monospace;
}

.panel-surface {
  background: linear-gradient(
    180deg,
    color-mix(in oklch, var(--card) 92%, white 8%) 0%,
    var(--card) 100%
  );
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: 0 1px 0 0 oklch(1 0 0 / 0.04) inset;
}
```

This is the exact pattern shadcn's `init` emits for Tailwind v4. Generated component CSS reads `var(--background)`; utilities resolve through the inline `@theme` mapping.

### 6.3 Theme toggle

- Library: `next-themes`.
- `<ThemeProvider attribute="class" defaultTheme="dark" enableSystem disableTransitionOnChange>` in `app/providers.tsx`. `attribute="class"` is required because shadcn's generated component CSS is scoped via `.dark` (see §6.2), not `[data-theme]`.
- `<ThemeToggle>` button (sun/moon icon from `lucide-react`) in `AppHeader`, between workspace switcher and avatar.
- First-load: respects OS preference. After toggle, persists to `localStorage`. Hydration-safe: `<html suppressHydrationWarning>` on the root.

### 6.4 Typography

- Inter for UI.
- JetBrains Mono for code/log/diff blocks in Agent Workspace, Review Room.
- Loaded via `next/font/local` from `public/fonts/`. **Reason:** `next/font/google` fetches font CSS at build time, which makes `pnpm build` and `pnpm tauri build` require network access — bad for offline desktop dev and CI air-gaps. Local font files are committed to the repo.
- Both fonts use `display: 'swap'` and CSS variables (`--font-inter`, `--font-jetbrains-mono`) wired into the `@theme inline` block above.

### 6.5 shadcn config (`components.json`)

- Style: `"new-york"`
- Base color: `"neutral"` (tokens above override)
- CSS variables: on
- RSC: off (static export, all client)
- TypeScript: on
- Icon library: `lucide-react`

### 6.6 Components installed in first pass

`button`, `card`, `badge`, `separator`, `scroll-area`, `tooltip`, `avatar`, `checkbox`, `tabs`, `dropdown-menu`. Additional components installed as panels need them during implementation.

---

## 7. Mock Data Strategy

`lib/mock-data.ts` is the canonical TS fixture file. `src-tauri/src/fixtures.rs` mirrors it (small enough that manual mirroring is fine for this scaffold). Both seed:

- 1 default project
- ~5 tasks across statuses (Draft, Running, Review Ready, Approved)
- 2 engines (Claude Code: ready; Codex: not_authenticated)
- 1 verification run with mixed pass/fail
- ~6 conversation messages
- ~4 active agent cards
- ~6 nodes for the Context Graph SVG

**Boundary types** (Task, Project, EngineStatus, VerificationRun, AppError) live in `src-tauri/src/models.rs` and flow into the frontend via the `tauri-specta`-generated `lib/bindings.ts`. **UI-only types** (component prop unions, store shapes, panel-local enums not crossing the IPC boundary) live in `lib/types.ts`. Field names in `models.rs` follow `docs/product-spec.md` and use `#[serde(rename_all = "camelCase")]` so the TS side reads `projectId`, `acceptanceCriteria`, etc.

---

## 8. Definition of Done

1. **Repo bootstrap** — `git init` runs only if `.git/` does not already exist at repo root (guarded — never reinitialize an existing repo). `.gitignore` covers Node + Rust + Tauri + macOS + Linux.
2. **`pnpm install`** — clean, no peer-dep warnings beyond known shadcn ones.
3. **`pnpm dev`** — Next.js renders the dashboard at `localhost:3000` with all 7 panels populated. Theme toggle works (dark↔light).
4. **`pnpm tauri dev`** — desktop window opens with the same UI; Rust command stubs are invoked through services and return fixtures consumed by the UI.
5. **`pnpm typecheck`** — passes (TS strict). Generated `lib/bindings.ts` from `tauri-specta` typechecks.
6. **`pnpm lint`** — passes (Next.js default ESLint config).
7. **`pnpm build`** — produces static `out/` without network access.
8. **`pnpm tauri build`** — smoke-builds a `.app` on macOS or an `.AppImage`/`.deb` on Linux (no signing). Build must succeed on whichever platform the developer runs it; CI matrix for both is a later concern.
9. **Visual fidelity** — at 1440×900, the running app is recognizably the same product as `ui.png`: 3-column layout, panel inventory, dark theme, glass surfaces, accent colors. Not pixel-perfect.

---

## 9. Implementation Sequence (preview)

The detailed plan comes from writing-plans, but rough order:

1. Repo + tooling: guarded `git init`, `pnpm init`, Next.js 15 + TS + Tailwind v4.
2. shadcn setup (two-layer tokens, `attribute="class"`), `next-themes` provider, theme toggle, local fonts.
3. Tauri v2 scaffold + `tauri.conf.json` + static export wiring + capabilities.
4. Rust skeleton: `error.rs`, `state.rs`, `models.rs` (with specta + serde camelCase), `fixtures.rs`, service modules (mock-backed), placeholder modules for deferred services, command thin-wrappers.
5. `tauri-specta` binding generation wired into `build.rs` or a Cargo test; check generated `lib/bindings.ts` into `.gitignore` or commit (decide during implementation).
6. `lib/tauri.ts` wraps generated bindings + dev-mode mock dispatcher (`isTauri()` detection).
7. Zustand stores + TanStack Query provider + feature hooks consuming the typed bindings.
8. `<DashboardShell>` + `<PanelFrame>` + `<AppHeader>` + `<ThemeToggle>`.
9. Panels in order: Task Board → Agent Workspace → Review Room → Context Graph → Conversation → Agent Manager → Engineering Snapshot.
10. Verification: `pnpm dev`, `pnpm tauri dev`, `pnpm build`, `pnpm tauri build` — all clean.

---

## 10. Risks & Mitigations

| Risk | Mitigation |
|---|---|
| Tauri v2 + Next.js App Router + static export rough edges | Constrain to the allowlist in §5.5 (no Route Handlers, Server Actions, middleware, ISR, request-time APIs, Node-only imports). |
| Tailwind v4 + shadcn compatibility (still evolving) | Use shadcn's documented v4 two-layer token pattern (§6.2) and `attribute="class"` for next-themes (§6.3). Pin known-good shadcn + Tailwind versions in `package.json`. |
| Rust toolchain not installed on dev machine | `pnpm tauri dev` will surface a clear error; document `rustup` requirement in README. |
| Boundary type drift between TS and Rust | `tauri-specta` generates `lib/bindings.ts` from Rust types at build time. TS types are not hand-written for the boundary. |
| Mock data drift between TS and Rust | Keep fixtures small. `fixtures.rs` carries a comment pointing to `lib/mock-data.ts` and the panel inventory; treat the TS file as canonical. |
| Context Graph network viz scope creep | Placeholder static SVG in this scaffold; real graph deferred. |
| OKLCH browser support | Modern Chromium (Tauri's webview) supports OKLCH natively; not a blocker for desktop. |
| `tauri-specta` codegen failure on first build | Document in README: if bindings file is out of sync, run the codegen step explicitly. Specta is widely used in Tauri v2 projects, but its plugin surface evolves — pin a known version. |
