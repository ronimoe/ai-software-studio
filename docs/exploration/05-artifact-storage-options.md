# Exploration: Artifact Storage Options

## Status

Exploring

## Question

Where should AI Software Studio store logs, diffs, verification results, and reports?

## Context

The app needs to store both structured metadata and large text artifacts.

Examples:

- terminal logs
- git diffs
- verification logs
- PR reports
- risk reports
- review summaries

## Options

### Option A: Store Everything in SQLite

#### Pros

- Simple backup
- Easy query
- Single database file

#### Cons

- Large logs can bloat database
- Harder to inspect manually
- Poor fit for large artifacts

### Option B: Store Metadata in SQLite, Files on Disk

#### Pros

- Keeps database small
- Artifacts are inspectable
- Good for logs and diffs
- Easy to delete/export

#### Cons

- Need to keep DB and files in sync
- More filesystem management

### Option C: Store Artifacts Inside Git Worktree

#### Pros

- Close to the code
- Easy to inspect

#### Cons

- Pollutes repo/worktree
- Risk of accidentally committing artifacts
- Not ideal

## Recommended Direction

Use:

```text
SQLite for metadata
Filesystem for artifact bodies
```

## Decision Trigger

Create ADR if the storage layout becomes stable.
