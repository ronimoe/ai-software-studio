# ADR-004: Use Local CLI Engine Adapters

## Status

Accepted

## Context

AI Software Studio should support Claude Code and Codex CLI without directly managing OpenAI or Anthropic API tokens.

The user wants each developer to use their own local engine installation and subscription where applicable.

The app should not become a provider API proxy in the initial architecture.

## Decision

Use **local CLI engine adapters**.

The app detects and controls local engine CLIs through a common adapter interface.

Initial adapters:

- Claude Code CLI adapter
- Codex CLI adapter

The engines handle their own authentication and provider usage.

## Consequences

### Positive

- No provider API tokens stored by the app
- Lower operating cost
- Works with user-owned subscriptions/accounts
- Keeps product provider-agnostic
- Enables future engines through adapter pattern

### Negative

- CLI behavior may be inconsistent
- Authentication state may be hard to detect
- Interactive prompts may require PTY
- Engine output may not be structured
- Breaking CLI changes can affect the app

## Alternatives Considered

### Direct Provider API Integration

Rejected for initial architecture because it requires token management, billing, model routing, and provider-specific API logic.

### Single Engine Only

Rejected because the product should be engine-agnostic.

### Manual External Terminal Only

Useful as fallback, but not enough for the core product UX.

## Revisit When

- Provider APIs offer better official agent integration
- Local CLI behavior is too unreliable
- MCP or another protocol becomes the better integration model
