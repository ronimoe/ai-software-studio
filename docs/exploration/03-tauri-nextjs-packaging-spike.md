# Exploration: Tauri + Next.js Packaging

## Status

Exploring

## Question

What is the cleanest way to package a Next.js UI inside a Tauri desktop app for macOS and Linux?

## Context

The selected stack is:

```text
Tauri + Rust native core + Next.js UI + SQLite
```

The app should run as a local desktop application, not as a hosted web app.

## Options

### Option A: Static Export

Build the Next.js app as static assets and let Tauri load them.

#### Pros

- Simple deployment
- No local Next.js server needed
- Good for desktop app packaging

#### Cons

- No server-side runtime features
- Need to avoid Next.js APIs that require a running server

### Option B: Local Next.js Server

Run a local Next.js server inside the desktop app.

#### Pros

- Supports more Next.js features
- Easier during early development

#### Cons

- More moving parts
- Harder packaging
- Slower startup
- More runtime failure points

### Option C: Vite React Instead of Next.js

Use Vite + React for the desktop UI.

#### Pros

- Simpler for Tauri
- Fast build
- Great for SPA desktop apps

#### Cons

- Loses Next.js app structure
- Less aligned with existing preference

## Recommended Direction

Use Next.js static export for the packaged app.

During development, use the Next.js dev server with Tauri pointing to localhost.

## Prototype Test

1. Create Tauri app.
2. Add Next.js UI.
3. Configure dev mode to load localhost.
4. Configure production mode to load static export.
5. Package macOS build.
6. Package Linux AppImage.
7. Confirm Tauri commands work in both modes.

## Decision Trigger

Create an ADR if static export is confirmed as the production packaging strategy.
