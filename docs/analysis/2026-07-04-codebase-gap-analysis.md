# Codebase Gap Analysis — The Trust Layer for Agent-Written Code

**Date:** 2026-07-04
**Codebase state:** v0.1.0 + auto-dispatch queue (commit `307f443`)
**Companion doc:** [2026-07-04-execution-plan.md](2026-07-04-execution-plan.md)

---

## 1. Strategic frame

**AI Software Studio is not an AI coding IDE, and it does not try to run agents better than the model vendors.** It is the **trust & evidence layer for agent-written code**: the neutral, engine-independent system that turns an agent's pull request into something a human — or an enterprise — can *audit and trust*. Every PR it produces is independently verified against stated acceptance criteria, pinned to an exact commit, risk-labeled by what it touched, policy-gated, and backed by a full transcript of what the agent actually did.

The desktop Studio is the *front-end* — where a human dispatches work and reviews the evidence. But the durable, defensible product is the **evidence itself and the attestation that produces it.** That distinction drives every priority in this analysis.

**Why this is the survivable position.** The autonomous-coding market is filling up: Cursor background agents, Claude Code cloud sessions, Codex cloud tasks, and worktree fleet managers (Conductor, Terragon, etc.) all converge on "run N agents in parallel, human reviews." Competing there — on who runs agents most nicely — is a race against companies with 100× the resources, and the host platform can absorb a thin wrapper at any release. Running agents is table stakes, not a moat.

What the model vendors structurally *will not* build well is **neutral, cross-engine attestation**. An engine grading its own homework is not evidence. A third party that can say "this PR was independently verified against these criteria, touched these risk surfaces, followed test-first discipline, and here is the proof — regardless of which model wrote it" has no natural owner among the providers, and is exactly what an organization needs before it lets agents write production code. That is the product.

**What the trust layer requires:** structured intent going in (the task brief), isolated execution (worktrees), independent verification, risk/policy labeling, commit-pinned evidence, a full run transcript, and a way to publish that evidence where merge decisions are actually made (the PR). We have shipped intent, isolation, verification, and a first-draft evidence report. We have **not** shipped the parts that make the evidence *trustworthy or portable*: risk/policy is an empty stub, TDD discipline is unenforced, transcripts are discarded, verification isn't pinned to a commit, engine-neutrality isn't real, and the evidence never leaves the desktop app. Those gaps are the analysis.

---

## 2. What actually works today (verified against code)

The v0.1 closed loop is real and shipped. A user can: open a repo → create a task with acceptance criteria and constraints → create an isolated worktree (with rollback on failure, `core/worktree_lifecycle.rs`) → run Claude Code (`claude --print`, `engines/adapters/claude_code.rs`) → watch live stdout/stderr in the terminal panel (Tauri events, `process/mod.rs`) → view changed files and unified diffs → run independent verification (install/typecheck/lint/test/build, per-project configurable) → generate a Markdown evidence report → push and open a GitHub PR via `gh`. The v0.2 auto-dispatch queue (`dispatch/worker.rs`) drains queued tasks through a 5-stage pipeline (worktree → agent → reconcile → verify → PR) with single retries, pause/resume, atomic dequeue, and orphan recovery on startup.

Engineering quality is genuinely good: seam-based testability (`dispatch/seams.rs`), compensating-action rollback, substantive Rust test coverage, CI gates, TDD discipline visible throughout. This is a solid foundation — the gaps below are about *scope*, not *rot*.

---

## 3. Gap inventory

### 3.1 Correctness gaps (verify + fix first — these undermine the pipeline that exists)

| # | Gap | Evidence | Why it matters |
|---|-----|----------|----------------|
| C1 | **Nobody commits.** The pipeline pushes and creates a PR, but no stage commits the agent's changes, and the agent prompt doesn't instruct it to. | `dispatch/worker.rs` stage 5 → `engines/github.rs` (`git push`, `gh pr create`); no commit anywhere | If the agent leaves changes uncommitted, `git push` pushes an empty branch and `gh pr create` fails ("no commits between base and head") → every autopilot run dead-ends at ReviewReady. |
| C2 | **Diff/status computed against worktree `HEAD`, not the base branch.** | `git/diff.rs` (`git diff HEAD -- <path>`), `git/status.rs` (porcelain) | Inverse failure of C1: if the agent *does* commit, `git status` is clean → reconcile stage reports "no-changes" → task marked Stopped, and the diff viewer shows nothing. Changed-files/diff must be computed vs. merge-base with the base ref. |
| C3 | **Agent run transcripts are not persisted.** Output streams to UI and is then lost; only 4KB verification excerpts survive. | `process/mod.rs` (events only), `artifacts/mod.rs` (only `task-brief.md` written) | An "evidence-backed workflow" that discards the primary evidence (what the agent actually did) can't support audits, debugging, or the review loop. |
| C4 | **Verification runs aren't tied to a commit SHA.** | `verification/repository.rs` schema | The evidence report can't prove *which code* passed verification. If anything changes after the run, the evidence is stale and nothing detects it. |

C1/C2 together mean the autopilot loop as designed likely cannot complete cleanly in either the committed or uncommitted case. This should be reproduced and fixed before anything else is built on top.

### 3.2 Backend gaps (stub or missing vs. spec/architecture promises)

| # | Gap | State | Promised in |
|---|-----|-------|-------------|
| B1 | **Policy/risk engine** — sensitive-path detection, dependency/migration/infra risk labels | `policy/mod.rs` is a 3-line TODO stub; `RiskLevel` enum exists in `models.rs` but is never computed | Spec §6.6, brief §9, roadmap v0.2 |
| B2 | **Project config** — `.aistudio/config.yaml` | `config/mod.rs` is a 3-line TODO stub; settings live only in SQLite | Spec §8, roadmap v0.2 |
| B3 | **Codex CLI adapter** — second engine | Rejected at dispatch with "no adapter yet" (`commands/dispatch.rs:26`) | ADR-004, spec §4, brief §3 |
| B4 | **Interactivity** — one-shot `claude --print` only; no session resume, no answering agent questions, no PTY | `engines/adapters/claude_code.rs`, `process/mod.rs` (pipes only) | Spec §6.5, roadmap v0.2 (NeedsInput) |
| B5 | **Structured agent output** — plain text pipe; no tool-call events, no token/cost data, no session id | `claude_code.rs` uses bare `--print` | (Enabler for conversation panel, telemetry, resume) |
| B6 | **Sequential-only dispatch** — one worker, insertion order, no concurrency, no priority | `dispatch/worker.rs` | Worktree isolation (ADR-005) exists precisely to enable parallelism |
| B7 | **Artifact store is skeletal** — no manifest, no lifecycle, no run outputs or diff snapshots | `artifacts/mod.rs` (34 lines, dir helper + brief only) | Architecture §8.2, exploration 05 |
| B8 | **Skills / agentic intake** — static CLAUDE.md priming only; intake is a form | `core/worktree_context.rs` | Roadmap v0.2/v0.3 |
| B9 | **Hard TDD verification** — red→green discipline is prompt-only, unenforced | (nothing) | Roadmap v0.2 (TDDVerifier) |

### 3.3 Frontend gaps

| # | Gap | State |
|---|-----|-------|
| F1 | **No approve/reject/request-changes UI.** ReviewReady is a dead end — the *human review verdict*, the core of the product loop, has no buttons and no backend mutation. Statuses `approved`/`changesRequested` exist only in mock data. | `components/panels/review-room/` |
| F2 | **3 of 7 panels are decorative mocks** — and they're the *supervision* panels: Conversation (static 6 messages, no input), Context Graph (static 6-node SVG), Engineering Snapshot (3 hardcoded metrics incl. a fake "spend" figure). | `features/conversation/`, `features/context-graph/`, `features/snapshot/` — all marked `TODO(phase-2)` |
| F3 | **No re-run/iterate loop.** After reviewing a diff there is no "send feedback and run again" path — the daily-driver workflow (review → request changes → agent iterates) doesn't exist. | (depends on F1 + B4/B5) |
| F4 | Agent Manager "Re-detect" button has no onClick handler. | `components/panels/agent-manager/index.tsx:24` |
| F5 | `task-store` activity log is static mock; live terminal bypasses it — two sources of truth for "what the agent did." | `stores/task-store.ts:19` |
| F6 | Stub hooks returning empty/mock: artifacts, worktrees list, active agents. | `features/artifacts/`, `features/worktrees/`, `features/engines/use-active-agents.ts` |

### 3.4 Strategic gaps (the trust-layer thesis — market-driven, not spec-driven)

| # | Gap | Bearing on the thesis |
|---|-----|---------------------|
| S1 | **The trust layer is unbuilt — this *is* the product, and it's a stub.** "Independently verified, risk-labeled, commit-pinned, transcript-backed" is the entire reason to exist versus a raw terminal. Verification exists; risk/policy (B1), TDD enforcement (B9), transcript evidence (C3), and commit-pinned verification (C4) do not. Today the evidence report is a nicely formatted *claim*, not proof. Closing this is existential; everything else serves it. |
| S2 | **Evidence is trapped in the desktop app.** The attestation is rendered into a PR *body* (Markdown prose) and nowhere else — not a GitHub status check, not a commit attestation, not a machine-readable artifact. Merge decisions happen in GitHub; if the evidence can't gate or annotate the PR where the team already works, the trust layer has no teeth. |
| S3 | **Engine-neutrality is not yet real.** One adapter (Claude Code), one gated-out stub (Codex). A *neutral* attestation layer that works with only one vendor's agent is not neutral — and is one vendor release from redundancy. A second engine is not a feature; it's proof the layer sits *above* the engines. |
| S4 | **Agents run with the user's full permissions.** A product whose pitch is "trust agent output" that itself executes untrusted agent actions with full filesystem/network access has a credibility gap. Sandboxing the run is part of the trust story, not an ops detail — and it graduates to urgent the moment anyone runs this on a repo they don't fully control. |
| S5 | **Steering + attention + fleet are throughput enablers, not the differentiator.** Parallel execution (B6), NeedsInput/resume (B4), and attention routing let the layer process *many* PRs and are table stakes for a daily driver — but they are the delivery mechanism, not the moat. Build them to feed the evidence layer, not as ends in themselves. |
| S6 | **Execution is laptop-bound.** A local-first desktop app physically caps fleet size and can't sit in CI, where autonomous PRs increasingly originate. This is a deliberate bet (a privacy/enterprise wedge) — but if the era moves server-side, the trust layer must be able to attest to runs it didn't host locally. Flag now, decide before v1.0. |
| S7 | **Cost/token telemetry is a non-goal that should be reversed.** "What did this cost" is part of trustworthy evidence, and it falls out of structured engine output for free (B5). Fold it into the attestation rather than treating it as out of scope. |

### 3.5 Hygiene (small, fix opportunistically)

- README badge says `0.0.4 — pre-MVP`; actual version is `0.1.0` (`VERSION`, `CHANGELOG.md`).
- `fixtures.rs` and `lib/mock-data.ts` must be kept in sync by hand (comment-enforced only) — drift risk as commands grow.
- Verification runs sequentially inside the dispatch pipeline; with parallel workers (S1), concurrent `pnpm install`s will contend — verification needs its own concurrency gate later.
- Every new command still needs a hand-written mock in `lib/tauri.ts` (`mockImpl`) — fine for now, but it's a growing tax; consider generating mocks from fixtures.

---

## 4. Assessment

**What's strong:** the skeleton is the right skeleton *for a trust layer*. The two-process split (dangerous work isolated in Rust), the engine-adapter seam (neutrality is possible), worktree isolation, independent verification, and the evidence-report framing are exactly the bones an attestation product needs. Code quality and test discipline are high. The gaps are scope, not rot.

**What's wrong, in one sentence:** the product has built the *machinery* of a trust layer (isolate → run → verify → report) but not the *trust* — the evidence going in is incomplete (no transcript, no commit pinning: C3/C4), the judgment on top of it doesn't exist (no risk/policy B1, no TDD enforcement B9), and the evidence coming out never leaves the desktop to reach the place merges are decided (S2).

**What "not an IDE" changes about priorities.** The earlier framing treated the trust layer as a late-phase moat to reach *after* parallelism and a slick supervision UX. Reversed: the trust layer **is** the product, so it comes right after the loop is made honest; the fleet, the conversation panel, and the attention board are the delivery surface built *around* it, not before it.

**Priority logic used in the execution plan:**
1. Make the existing loop true and capture real evidence (C1–C4) — you cannot attest to evidence you don't reliably have.
2. Build the trust layer (risk/policy, TDD verifier, commit-pinned + transcript-backed evidence) — the product itself.
3. Give the evidence teeth: publish it where merges happen (GitHub status checks / PR annotations / machine-readable artifact) and wire the human verdict loop.
4. Make the agent seam structured, steerable, and multi-engine — richer evidence and *real* engine-neutrality (S3).
5. Scale throughput: parallel fleet + attention routing — process many PRs, feed the layer (S5).
6. Compounding (skills, intake) plus the forward bets — sandboxed execution (S4) and remote/CI attestation (S6) — for durability.

See the [execution plan](2026-07-04-execution-plan.md) for phases, scope cuts, and sequencing.
