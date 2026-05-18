# Exploration: Terminal and PTY Options

## Status

Exploring

## Question

What is the best way to run interactive local coding agents such as Claude Code and Codex CLI inside AI Software Studio?

## Context

The app needs to start local CLI-based agents, stream their output, allow user input, and stop them safely.

Simple process spawning may not be enough if the agent expects an interactive terminal.

## Options

### Option A: Simple Process Runner

Use standard process spawning with stdout, stderr, and stdin.

#### Pros

- Simpler to implement
- Easier to debug
- Good for non-interactive commands

#### Cons

- May fail with interactive CLIs
- May not support rich terminal behavior
- Harder to handle approval prompts

### Option B: Unix PTY

Use a pseudo-terminal for agent sessions.

#### Pros

- Better compatibility with interactive CLIs
- Supports terminal-like behavior
- Better for Claude Code / Codex-style workflows

#### Cons

- More complex implementation
- Requires careful process cleanup
- Output parsing may be noisier

### Option C: External Terminal Fallback

Open the task worktree in the user's terminal.

#### Pros

- Very reliable fallback
- Minimal implementation complexity
- Useful when embedded PTY fails

#### Cons

- Less integrated UX
- Harder to capture full output
- Less controlled by the app

## Recommended Direction

Use Unix PTY as the primary implementation and provide external terminal fallback.

## Prototype Test

1. Spawn Claude Code inside a Git worktree.
2. Stream output to the UI.
3. Send user input back to the process.
4. Stop the process with SIGTERM.
5. Force stop with SIGKILL if needed.
6. Save full terminal output as an artifact.

## Decision Trigger

Create an ADR when Unix PTY is confirmed stable enough for MVP.
