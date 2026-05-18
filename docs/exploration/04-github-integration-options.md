# Exploration: GitHub Integration Options

## Status

Exploring

## Question

How should AI Software Studio integrate with GitHub?

## Context

The MVP can work without deep GitHub integration. However, GitHub integration becomes useful for:

- importing issues
- creating branches
- pushing branches
- creating pull requests
- adding PR evidence reports
- linking task status to PRs

## Options

### Option A: Manual PR Report Only

Generate Markdown report and let user manually create PR.

#### Pros

- Simplest
- No OAuth required
- Good for local-first MVP
- Avoids cloud complexity

#### Cons

- More manual steps
- Less polished workflow

### Option B: GitHub CLI Integration

Use the user's existing `gh` CLI authentication.

#### Pros

- No app-managed GitHub OAuth initially
- User owns authentication
- Very aligned with local-first model
- Easy to create PRs from local branches

#### Cons

- Requires `gh` installed
- Need to handle CLI errors
- Less control than GitHub API

### Option C: GitHub OAuth App

User logs into GitHub through the app.

#### Pros

- Polished UX
- Better control over GitHub features
- Can import issues and create PRs directly

#### Cons

- More auth complexity
- Need secure token storage
- More setup

### Option D: GitHub App

Best for team/enterprise later.

#### Pros

- Fine-grained permissions
- Better for organization use
- Good auditability

#### Cons

- Too much for personal/local MVP

## Recommended Direction

MVP:

```text
Manual PR report + optional GitHub CLI support
```

Later:

```text
GitHub OAuth or GitHub App
```

## Decision Trigger

Create an ADR when deciding whether GitHub CLI is part of MVP.
