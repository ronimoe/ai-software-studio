# ADR-003: Support macOS and Linux Only Initially

## Status

Accepted

## Context

AI Software Studio needs to run local agent CLIs, manage terminals, create Git worktrees, and run project commands.

Supporting Windows adds complexity around:

- PowerShell and cmd behavior
- path differences
- ConPTY
- WSL ambiguity
- Git for Windows differences
- process tree termination
- CLI compatibility differences

## Decision

Support **macOS and Linux only** for the initial product.

Windows support is intentionally excluded from the first version.

The app assumes a Unix-like environment with:

- POSIX paths
- Unix signals
- bash/zsh/sh-style shell environments
- Unix PTY support

## Consequences

### Positive

- Simpler terminal implementation
- Easier process control
- Easier path handling
- More predictable CLI behavior
- Faster MVP and V1 development
- Better fit for many developer workflows

### Negative

- Windows users cannot use the first release
- Future Windows support may require architectural additions
- Some teams may need Windows support before adoption

## Alternatives Considered

### Full Cross-Platform Support From Day One

Rejected because Windows adds too much complexity for the initial product.

### WSL-Only Windows Support

Deferred. It may be considered later if there is demand.

## Revisit When

- macOS/Linux workflow is stable
- user demand for Windows is clear
- PTY/process abstraction is mature
- WSL integration becomes a product priority
