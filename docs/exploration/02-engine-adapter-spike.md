# Exploration: Engine Adapter Spike

## Status

Exploring

## Question

How should AI Software Studio abstract local coding engines such as Claude Code and Codex CLI?

## Context

The app should support multiple local agents without coupling the product to one provider or CLI.

The first engines are:

- Claude Code CLI
- Codex CLI

Future engines may include:

- Gemini CLI
- Aider
- OpenCode
- local models
- custom enterprise agents

## Proposed Adapter Interface

```rust
pub trait EngineAdapter {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;

    async fn detect(&self) -> EngineDetectionResult;
    async fn health_check(&self) -> EngineHealthResult;
    async fn start_task(&self, input: StartTaskInput) -> Result<EngineRunHandle>;
    async fn send_message(&self, run_id: String, message: String) -> Result<()>;
    async fn stop(&self, run_id: String) -> Result<()>;
}
```

## Key Unknowns

- Can each engine run reliably in a controlled PTY?
- Can sessions be resumed?
- Can we detect authentication state?
- Can we pass structured prompts reliably?
- Can we detect when the engine is waiting for approval?
- Can we safely stop an agent without corrupting the worktree?

## Recommended Direction

Use a common adapter interface, but keep engine-specific behavior isolated.

Do not over-normalize too early. The first adapter should capture raw terminal logs and rely on Git diff and verification as the source of truth.

## Decision Trigger

Create an ADR when the adapter interface is proven against at least two local engines.
