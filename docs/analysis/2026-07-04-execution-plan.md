# Execution Plan — Building the Trust Layer for Agent-Written Code

**Date:** 2026-07-04
**Input:** [2026-07-04-codebase-gap-analysis.md](2026-07-04-codebase-gap-analysis.md)
**Positioning:** this product is the **neutral, engine-independent trust & evidence layer** for agent-written code — not an AI coding IDE, and not an attempt to run agents better than the model vendors. The desktop Studio is the front-end; the durable product is the attestation. Every phase below is judged by whether it makes the evidence more *trustworthy*, more *portable*, or produced at greater *throughput* — in that priority order.
**Convention:** each numbered plan is sized to become one `docs/superpowers/plans/` file and one PR-sized unit of work, TDD throughout (failing test first, per CLAUDE.md). No code in this document.

Gap IDs (C1, B1, S2 …) refer to the gap analysis tables.

---

## Phase A — Make the loop true & capture real evidence

*Goal: the pipeline that already ships must complete cleanly and leave behind evidence worth attesting to. You cannot build a trust layer on evidence you don't reliably have. Everything later inherits this.*

| Plan | Scope | Gaps | Size |
|------|-------|------|------|
| **A1. Base-ref diffing** | Compute changed files and diffs against `merge-base(base_ref, HEAD)` instead of worktree `HEAD`, so committed and uncommitted agent work both show correctly in reconcile, review room, and the evidence report. Reproduce both C1/C2 failure modes as failing Rust tests first. | C2 | S |
| **A2. Commit stage** | Add an explicit commit step to the pipeline (and manual flow): if the worktree is dirty after the agent exits, commit as a labeled checkpoint (`aistudio: task <id>`) before verification. Guarantees push/PR always has commits and that verification runs against a pinned tree. | C1 | S |
| **A3. Run transcript persistence** | Tee `TaskOutput` lines to a per-run artifact file (`artifacts/<task>/runs/<run-id>.log` or JSONL), add a run manifest, expose `listArtifacts`; wire the stub hook (`features/artifacts`). The activity log reads from this instead of the static mock. This transcript is the primary evidence of what the agent did. | C3, B7, F5, F6 | M |
| **A4. Commit-pinned verification** | Store the commit SHA on every verification run; render it in the evidence report; flag stale evidence in the Review Room when worktree SHA ≠ verified SHA. Attestation must name the exact code it verified. | C4 | S |
| **A5. Hygiene sweep** | README/version badge, Agent Manager re-detect onClick, remove or clearly label the fake "spend" metric until Phase D delivers real telemetry. | F4, hygiene | XS |

**Exit criteria:** an autopilot run (queue → PR) completes with a real PR containing the agent's commits, a diff that matches what the reviewer saw, a persisted transcript, and an evidence report pinned to a SHA.

---

## Phase B — The trust layer (the product)

*Goal: turn the evidence report from a formatted claim into proof. This is what the company sells. It depends only on Phase A — not on the fleet, the fancy seam, or a slick UI — which is exactly why it moves to the front.*

| Plan | Scope | Gaps | Size |
|------|-------|------|------|
| **B1. Policy engine v1** | Implement `policy/`: glob-based sensitive-path rules (secrets, CI config, infra, migrations, lockfiles) + dependency-manifest change detection → compute `RiskLevel` per task from its changed files. Surface as badges on changed files, task cards, and the evidence report. | B1(gap), S1 | M |
| **B2. Risk-gated autopilot** | Dispatch consults policy: high-risk changes stop before the PR stage and require explicit human approval even in autopilot. The headline trust feature — *agents run free on safe changes; humans gate risky ones.* | B1(gap), S1 | S (on B1) |
| **B3. Hard TDD verifier** | Scan worktree commit history for red→green evidence (a test-touching commit precedes/accompanies implementation); record as a check in the evidence report. Ship informational first, gating later. This is attestation the vendors can't self-certify. | B9, S1 | M |
| **B4. Project config file** | Implement `config/`: `.aistudio/config.yaml` for verification commands, policy globs, engine preference, concurrency — versioned with the repo, overriding DB defaults. Makes the *rules* of attestation themselves reviewable and shareable per project. | B2(gap) | M |
| **B5. Evidence bundle spec** | Define the canonical evidence object: task + criteria satisfaction + pinned SHA + verification results + risk label + TDD check + transcript reference + cost. One machine-readable schema (JSON) that the Markdown report and Phase C's GitHub publishing both render from. This schema *is* the product's contract. | S1, S2 | S |

**Exit criteria:** the evidence report shows risk label + TDD check + pinned SHA + criteria satisfaction, is emitted as a structured bundle (B5), and risky tasks cannot auto-PR (B2).

---

## Phase C — Give the evidence teeth (publish it + human verdict)

*Goal: get the attestation out of the desktop app and to the place merge decisions are made, and let a reviewer act on it. Evidence that never reaches GitHub can't gate anything.*

| Plan | Scope | Gaps | Size |
|------|-------|------|------|
| **C1. GitHub-native evidence** | Publish the evidence bundle (B5) as a **GitHub commit status / check run** on the PR head SHA (pass/neutral/fail from verification + risk gate), plus a structured PR annotation — not just prose in the PR body. The attestation now gates and annotates where the team already works. | S2 | M |
| **C2. Review verdict** | Approve / Request changes / Reject in the Review Room, backed by real status-transition commands with eligibility guards (mirroring the enqueue/stop guard pattern). Approved → PR; Rejected → worktree/branch cleanup offer. | F1 | M |
| **C3. Feedback → re-dispatch** | "Request changes" captures reviewer feedback, appends it to the task brief / worktree context, and re-enqueues the task. Completes the iterate loop even with a one-shot run; gets cheaper with Phase D resume. | F3 | M |
| **C4. Honest panels** | Repurpose the Conversation panel as the run-transcript view (from A3) until real conversation lands in Phase D; collapse the Context Graph behind a "preview" flag instead of showing fake data. Fake supervision surfaces cost trust with exactly the users a *trust product* courts. | F2 (partial) | S |

**Exit criteria:** a completed task posts a passing/failing check to its GitHub PR, and a human can approve/request-changes → agent re-runs → re-review, entirely in-app.

---

## Phase D — Structured, steerable, multi-engine seam

*Goal: richer evidence and real engine-neutrality. Replace the blind one-shot pipe with structured, resumable sessions — and prove the layer sits above any single vendor.*

| Plan | Scope | Gaps | Size |
|------|-------|------|------|
| **D1. Structured streaming adapter** | Move the Claude Code adapter to headless structured output (`--output-format stream-json`): parse assistant text, tool-use events, result/usage records, session id. Persist as the canonical transcript (upgrades A3 from raw lines to structured events feeding the evidence bundle). Keep raw-pipe mode as fallback behind the trait. | B5(gap) | M |
| **D2. Codex CLI adapter** | Second engine through the same seam (`codex exec` headless + JSON), engine selection honored end-to-end, dispatch gate removed. **This is the neutrality proof — do not market "engine-independent" until it ships.** | B3(gap), S3 | M–L |
| **D3. Real conversation panel** | Render structured events as a conversation; an input box issues a follow-up turn via session resume — steering *between* turns is sufficient; mid-turn interruption stays out of scope. | F2, B4(gap) | M |
| **D4. NeedsInput state** | Detect runs that end awaiting clarification; a new task status pauses the pipeline, surfaces the question, resumes on human answer. The worker treats NeedsInput as a yield, not a failure. | S5, B4(gap) | M |
| **D5. Cost/token telemetry** | Capture usage/cost from structured results per run into the evidence bundle; data source for the real snapshot (E3). Reverse the spec's "no cost tracking" non-goal. | S7 | S |

**Exit criteria:** a task can be dispatched on **either engine**, ask a question, receive an answer, finish, and record a structured transcript + cost in its evidence bundle.

---

## Phase E — Scale throughput (parallel fleet + attention)

*Goal: process many PRs so the trust layer operates at volume. Deliberately after B–D: a fleet of unattested, non-steerable agents just parallelizes low-trust output. Throughput is table stakes, not the moat.*

| Plan | Scope | Gaps | Size |
|------|-------|------|------|
| **E1. Concurrent dispatch** | Worker pool with configurable concurrency (default 2–3). The atomic CAS dequeue already supports this; add per-stage gates (agents parallel; verification serialized/bounded to avoid `pnpm install` contention). Extend orphan sweep for multiple in-flight tasks. | S5, B6 | L |
| **E2. Fleet board + attention queue** | Task board becomes the fleet view: live status chips from the `DispatchEvent` stream, plus an attention queue ordered "needs input → needs review → risk-gated → failed → running." With N agents, routing the human's attention is what keeps the trust layer usable. | S5 | M |
| **E3. Real engineering snapshot** | Replace mock metrics with DB aggregates: tasks in flight, queue depth, verification pass rate, spend (from D5), throughput (tasks → merged PRs), and **% merged on evidence alone** (the trust metric). | F2, S7 | S |
| **E4. Queue controls** | Priority ordering, UI reorder, per-task retry policy; fleet-level cancel-all / pause-all. | B6 | S–M |

**Exit criteria:** 3 tasks run concurrently → attention queue routes the human through input-requests, risk gates, and reviews → snapshot shows real throughput, spend, and evidence-alone merge rate.

---

## Phase F — Compounding + forward bets

*Goal: make each task cheaper, and de-risk the two bets that decide long-term survival — credibility (sandboxing) and deployment surface (server-side attestation).*

| Plan | Scope | Gaps | Size |
|------|-------|------|------|
| **F1. Skills packs** | Bundled instruction Markdown per task type (bugfix, feature, refactor, upgrade) injected into worktree context; per-project custom skills in `.aistudio/skills/`. | B8 | M |
| **F2. Agentic intake** | "Interview me" task creation: an agent session turns a rough goal into criteria/constraints/files-to-touch; human confirms. Reuses the seam from Phase D. | B8 | M |
| **F3. Context graph (real or cut)** | Derive a real graph (changed files ↔ criteria ↔ verification ↔ risk) from data that now exists, or delete the panel. No third option — decorative panels are debt for a trust product. | F2 | S–M |
| **F4. Sandboxed execution** | Run agents under filesystem/network confinement (container or OS sandbox) so "trust our output" isn't undercut by "we ran untrusted actions with your full permissions." **Graduates to high priority the moment the product targets repos the user doesn't fully control or enterprise buyers.** | S4 | L |
| **F5. Remote / CI attestation spike** | Exploration (an ADR, not code yet): can the evidence layer attest to agent runs it did *not* host locally — a CI job, a cloud sandbox? Decides whether local-first stays the wedge or becomes the ceiling. Resolve before committing v1.0 scope. | S6 | S (spike) |

---

## Sequencing, risks, and metrics

**Order:** A → B → C → D → E, with F last. A makes the evidence real; **B turns it into the product**; C gives it teeth in GitHub; D and E build the studio and fleet that produce evidence richly and at volume. B depends only on A — not on the fleet or the structured seam — which is the whole point of the reorder: the trust layer ships before the throughput machinery, because the trust layer is *why anyone would use the throughput machinery*.

**Key risks**
1. **Engine CLI churn** — headless flags/output formats drift across CLI versions. Mitigate: version-gate in the adapter, keep raw-pipe fallback, pin tested versions in detection.
2. **Parallel resource contention** — concurrent verifications can thrash a laptop. Mitigate: bounded verification concurrency (E1), configurable caps (B4), and F5's remote-execution question.
3. **Scope gravity toward "run agents better."** Every phase tempts toward competing with Anthropic/OpenAI on agent-running UX. The line: *we do not try to run agents better than the vendors; we attest to what any agent produced.* Anything that only makes the agent nicer to run — without strengthening or distributing the evidence — is out or deprioritized.
4. **Neutrality debt.** Shipping only a Claude adapter makes "neutral trust layer" a claim we can't back and a wrapper the host can absorb. Mitigate: land the second engine (D2) before any neutrality messaging; keep the adapter seam strict.
5. **Platform absorption.** The engine vendors are building their own run-and-review surfaces. Insurance is depth in the one thing they won't build — neutral, cross-engine, gated attestation — and portability of that evidence into GitHub (C1). The further we are from "a nicer terminal for Claude Code," the safer.
6. **Fixture drift** — every new command widens the hand-maintained mock surface (`lib/tauri.ts`, `mock-data.ts`, `fixtures.rs`). Consider a generation/lint step during Phase D when command count jumps.

**North-star metrics** (trust first, throughput second)
- **% of agent PRs merged with only the app's evidence reviewed** — no independent re-checking. The core trust metric; drives everything.
- **Evidence-bundle completeness** — transcript + commit-pinned verification + risk label + TDD check present on 100% of PRs.
- **Evidence portability** — attestation posted as a GitHub check/annotation on 100% of PRs.
- **Neutrality** — second engine reaches feature parity through the same seam.
- **Throughput** — concurrent tasks in flight (target ≥3 by end of E), reviewer minutes per merged PR, and % of safe-risk tasks reaching PR with zero human intervention.
