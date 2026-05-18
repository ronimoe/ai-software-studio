# Product Brief: AI Software Studio

## 1. Product Name

**AI Software Studio**

Alternative positioning names:

| Name | Positioning |
|---|---|
| AI Software Studio | Broad, product-friendly, good for solo developers and teams |
| AI Agent Cockpit | Strong local-agent orchestration positioning |
| Code Control Room | Memorable, but less formal |
| Agentic IDE | Technical, category-defining |
| AI Development Command Center | Clear differentiation from normal IDEs |

Recommended name:

> **AI Software Studio — the local command center for human-approved, evidence-backed AI coding agents.**

---

## 2. One-Line Summary

**AI Software Studio is a local-first desktop application that lets developers assign coding tasks to local AI agents such as Claude Code and Codex CLI, monitor their work, verify results, review evidence, and approve changes safely.**

---

## 3. Product Vision

Modern AI coding tools are powerful enough to write, edit, debug, and test code. But the workflow around them is still messy.

Developers often work through raw terminal sessions, chat windows, editor sidebars, and manual Git inspection. Important decisions are buried in logs. Verification is scattered. Review depends on manually inspecting diffs after the agent has already made changes.

AI Software Studio turns local AI coding agents into a structured engineering workflow.

The key shift:

```text
Old workflow:
Human asks AI to edit code
→ AI modifies files
→ human manually inspects everything

AI Software Studio:
Human defines task and constraints
→ agent works in isolated worktree
→ app captures output, diff, tests, and risks
→ human reviews evidence
→ human approves or rejects
```

The product does not try to replace Claude Code or Codex CLI. It wraps them with workflow, evidence, safety, and review.

---

## 4. Why This Product Should Exist

AI coding agents are becoming more useful, but adoption creates new problems:

| Problem | Description |
|---|---|
| Unstructured workflow | Coding agents are often driven through raw chat or terminal sessions |
| Low visibility | Developers may not clearly see what changed and why |
| Weak evidence | Agent claims are not enough; tests and diffs need to be captured independently |
| Risky changes | Agents can touch sensitive files such as auth, billing, infra, or migrations |
| Poor repeatability | Good prompts, constraints, and decisions are not stored as structured workflow artifacts |
| Review overload | AI can create changes faster than humans can review them |
| Token/cost concern | A hosted SaaS that pays for all model usage can become expensive |

The core opportunity:

> Developers already have local AI coding agents. What they lack is a safe workflow layer around those agents.

---

## 5. Target Users

## Primary User

Solo developer, technical founder, CTO, AI consultant, or senior engineer using:

- Claude Code
- Codex CLI
- Cursor
- VS Code
- Git
- local terminal
- GitHub

## User Goals

The user wants to:

- delegate implementation work
- keep control over decisions
- avoid uncontrolled code changes
- review diffs faster
- run verification consistently
- generate useful PR summaries
- avoid building a token-heavy hosted model wrapper

---

## 6. Product Positioning

AI Software Studio is not another AI chatbot or code autocomplete tool.

It is:

> **The missing workflow layer for local AI coding agents.**

## Positioning Statement

For developers using local AI coding agents, **AI Software Studio** is a local-first command center that structures agent work into tasks, constraints, isolated worktrees, verification, review, and PR evidence reports.

Unlike hosted AI coding platforms that proxy model usage, AI Software Studio uses the developer’s own local Claude Code or Codex CLI environment, avoiding central provider token storage and reducing platform-side model cost.

---

## 7. Core Differentiation

| Dimension | AI Chat / Terminal | AI Code Editor | AI Software Studio |
|---|---|---|---|
| Primary object | Prompt/session | File/editor | Task |
| Execution | Ad hoc | Editor-driven | Isolated worktree |
| Engine | One assistant | Usually one editor agent | Multiple local engine adapters |
| Verification | Manual | Partial | First-class verification dashboard |
| Review | Manual diff | Editor diff | Review room with evidence |
| Storage | Chat history/logs | Editor history | Structured local task history |
| Token model | Provider/user dependent | Provider/editor dependent | User-owned local engine usage |
| Human control | Prompt-based | Prompt/editor actions | Constraints, approvals, risk gates |

---

## 8. Product Principles

## 8.1 Local-First

Core functionality should work without a hosted backend.

## 8.2 No Provider Token Storage

The product should not store OpenAI or Anthropic API tokens in the initial architecture. Claude Code and Codex CLI handle their own authentication.

## 8.3 Human-Approved

Agents can execute, but humans own the final decision.

## 8.4 Evidence-Backed

The product should verify agent output independently through Git diffs, test runs, logs, and reports.

## 8.5 Isolated by Default

Agent work should happen in a dedicated Git worktree, not directly in the user’s active working tree.

## 8.6 Engine-Agnostic

Claude Code and Codex CLI are the first supported engines, but the adapter design should allow future engines.

---

## 9. Core Product Modules

| Module | Purpose |
|---|---|
| Project Dashboard | Open and manage local repositories |
| Engine Detection | Detect Claude Code, Codex CLI, Git, Node, package managers |
| Task Board | Create and manage structured coding tasks |
| Agent Workspace | Run a selected engine inside an isolated worktree |
| Command / Conversation Panel | Communicate with local agents and record instructions |
| Terminal Activity View | Stream and store terminal output |
| Changed Files Panel | Show changed files and risk labels |
| Verification Dashboard | Run tests, lint, typecheck, build, and custom commands |
| Review Room | Review summary, diff, evidence, risks, and decisions |
| Artifact Store | Store logs, diffs, reports, verification outputs locally |
| PR Report Generator | Generate Markdown evidence reports for pull requests |

---

## 10. MVP Scope

The MVP should prove the core workflow:

```text
Open repo
→ create task
→ create worktree
→ run Claude Code or Codex CLI
→ capture terminal output
→ show changed files
→ run verification
→ review diff and evidence
→ generate PR report
```

## Must Have

- Local repo picker
- Engine detection
- Task creation
- Acceptance criteria
- Constraints
- Git worktree creation
- Engine run through local CLI
- Terminal output streaming
- Changed file detection
- Git diff viewer
- Verification command runner
- Review room
- PR evidence Markdown generator
- Local SQLite storage

## Not MVP

- Windows support
- Hosted cloud execution
- Direct OpenAI/Anthropic API usage
- Multi-user collaboration
- Automatic production deployment
- Full visual context graph
- Deep GitHub App integration
- Enterprise policy system

---

## 11. Success Criteria

The MVP is useful if it makes local agent work more controllable and reviewable than raw terminal usage.

Potential success metrics:

| Metric | Target |
|---|---|
| Time from task to review-ready diff | Reduced by 30% |
| Manual copy/paste between tools | Reduced significantly |
| PR report usefulness | Report can be reused directly |
| Agent run visibility | User can inspect all output and changes |
| Review confidence | User can approve or reject faster |
| Recovery | Failed runs can be inspected and rerun |

---

## 12. Strategic Wedge

Do not position the first product as a full AI IDE replacement.

Position it as:

> **A local agent cockpit for Claude Code and Codex CLI.**

This avoids competing directly with VS Code, Cursor, or full cloud coding platforms.

The first job is not to replace the editor. The first job is to make local AI agent work structured, safe, and reviewable.

---

## 13. Long-Term Direction

Future expansion can include:

- GitHub issue import
- GitHub CLI PR creation
- MCP integration
- more engine adapters
- browser/screenshot verification
- team sync
- optional cloud dashboard
- enterprise policy templates
- hosted runners
- multi-agent orchestration

The long-term product could become a full AI-first software command center.

The first product should stay focused:

> Local-first workflow and evidence layer for existing AI coding agents.

---

## 14. Final Product Sentence

> **AI Software Studio is a local-first command center that turns Claude Code and Codex CLI into a structured, evidence-backed software development workflow.**
