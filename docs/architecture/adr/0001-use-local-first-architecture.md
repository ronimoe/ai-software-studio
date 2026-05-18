# ADR-001: Use Local-First Architecture

## Status

Accepted

## Context

AI Software Studio is intended to orchestrate local AI coding agents such as Claude Code and Codex CLI.

The user does not want a token-heavy hosted platform where the product pays for every model call. The product should use each developer's local engine installation and provider account.

The app also handles source code, terminal logs, diffs, and task artifacts, which are sensitive by default.

## Decision

Use a **local-first architecture**.

The core application runs on the user's machine. Agent execution, Git worktrees, verification commands, logs, diffs, reports, and local database storage are handled locally.

The app does not directly call OpenAI or Anthropic APIs in the initial architecture.

## Consequences

### Positive

- Avoids central model API costs
- Avoids storing provider API tokens
- Keeps source code and logs local by default
- Works with the user's existing Claude Code or Codex setup
- Reduces backend infrastructure requirements
- Fits solo developer and technical founder workflows

### Negative

- Harder to provide team collaboration initially
- Harder to collect centralized analytics
- Local environment differences can cause support issues
- Users must install required local tools

## Alternatives Considered

### Hosted SaaS Agent Runtime

A cloud backend could run agents and manage model credentials.

Rejected for now because it increases cost, token management, infrastructure burden, and trust concerns.

### Hybrid Cloud-First

A cloud dashboard could coordinate local agents.

Deferred until local workflow is proven.

## Revisit When

- Team collaboration becomes a primary product requirement
- Hosted runners become commercially necessary
- Enterprise customers require centralized governance
- Local agent execution becomes too inconsistent across environments
