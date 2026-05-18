# AI Software Studio — App Scaffold Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stand up the AI Software Studio monorepo (Tauri v2 + Rust + Next.js 15) with all 7 dashboard panels from `ui.png` rendered via mock data flowing through a typed `tauri-specta` boundary.

**Architecture:** Flat repo with Next.js 15 App Router at root (`app/`, `components/`, `lib/`, etc.) and Tauri/Rust at `src-tauri/`. Static-export Next config so Tauri loads the built output. Rust commands are thin wrappers over services (mock-backed today, real-backed later); `tauri-specta` generates TypeScript bindings into `lib/bindings.ts`. State is split between TanStack Query (server data) and Zustand (UI cursors).

**Tech Stack:** Tauri v2 · Rust (edition 2021) · Next.js 15 (App Router, static export) · TypeScript (strict) · Tailwind v4 (CSS-first, no `tailwind.config.ts`) · shadcn/ui (new-york, two-layer tokens) · next-themes · Zustand · TanStack Query · `tauri-specta` · pnpm.

**Spec:** [docs/superpowers/specs/2026-05-17-app-scaffold-design.md](../specs/2026-05-17-app-scaffold-design.md)

---

## File Structure Overview

Top-level files this plan creates (the repo currently has only `README.md`, `ui.png`, and `docs/`):

```
ai-software-studio/
├── .gitignore                          (Task 1)
├── package.json                        (Task 2)
├── tsconfig.json                       (Task 4)
├── next.config.ts                      (Task 4)
├── postcss.config.mjs                  (Task 7)
├── eslint.config.mjs                   (Task 5)
├── components.json                     (Task 9)
├── app/
│   ├── layout.tsx                      (Task 13)
│   ├── page.tsx                        (Task 13, rewritten Task 40)
│   ├── globals.css                     (Task 8)
│   └── providers.tsx                   (Task 11, extended Task 34)
├── components/
│   ├── ui/                             (Task 10 — shadcn batch install)
│   ├── layout/
│   │   ├── theme-toggle.tsx            (Task 12)
│   │   ├── panel-frame.tsx             (Task 37)
│   │   ├── app-header.tsx              (Task 38)
│   │   └── dashboard-shell.tsx         (Task 39)
│   └── panels/
│       ├── task-board/index.tsx        (Task 42)
│       ├── task-board/task-card.tsx    (Task 42)
│       ├── engineering-snapshot/...    (Task 43)
│       ├── agent-workspace/...         (Task 44)
│       ├── review-room/...             (Task 45)
│       ├── context-graph/...           (Task 46)
│       ├── conversation/...            (Task 47)
│       └── agent-manager/...           (Task 48)
├── features/
│   ├── tasks/use-tasks.ts              (Task 35)
│   ├── projects/use-projects.ts        (Task 35)
│   ├── engines/use-engines.ts          (Task 35)
│   └── verification/use-verification.ts(Task 35)
├── lib/
│   ├── bindings.ts                     (Task 25 — generated)
│   ├── tauri.ts                        (Task 29)
│   ├── mock-data.ts                    (Task 27)
│   ├── types.ts                        (Task 26)
│   └── utils.ts                        (Task 28)
├── stores/
│   ├── ui-store.ts                     (Task 31)
│   ├── task-store.ts                   (Task 32)
│   └── engine-store.ts                 (Task 33)
├── public/
│   └── fonts/                          (Task 11 — Inter + JetBrains Mono)
├── src-tauri/
│   ├── Cargo.toml                      (Task 14)
│   ├── build.rs                        (Task 14)
│   ├── tauri.conf.json                 (Task 14)
│   ├── capabilities/default.json       (Task 15)
│   ├── icons/                          (Task 15)
│   └── src/
│       ├── main.rs                     (Task 24)
│       ├── lib.rs                      (Task 24)
│       ├── error.rs                    (Task 17)
│       ├── models.rs                   (Task 18)
│       ├── fixtures.rs                 (Task 19)
│       ├── state.rs                    (Task 22)
│       ├── commands/{mod,projects,tasks,engines,verification}.rs (Task 23)
│       ├── engines/mod.rs              (Task 20 — service)
│       ├── tasks/mod.rs                (Task 20 — service)
│       ├── projects/mod.rs             (Task 20 — service)
│       ├── verification/mod.rs         (Task 20 — service)
│       └── core,db,git,process,policy,artifacts,config/mod.rs (Task 21 — placeholders)
└── README.md                           (existing; appended Task 52)
```

---

## Task 1: Initialize git repo (guarded) and write `.gitignore`

**Files:**
- Create: `.gitignore`

- [x] **Step 1: Verify we're at the project root and that no git repo exists yet**

Run: `pwd && ls -la .git 2>&1 | head -3`
Expected: `pwd` outputs `/Users/ronimoe/Development/ai-software-studio` and `ls -la .git` says "No such file or directory". If `.git` exists, SKIP `git init` in Step 2.

- [x] **Step 2: Guarded git init**

```bash
if [ ! -d .git ]; then git init -b main; fi
```

Expected: Either "Initialized empty Git repository …" or no output (if guarded out).

- [x] **Step 3: Write `.gitignore`**

Create `.gitignore` with this exact content:

```gitignore
# Node
node_modules/
.next/
out/
.pnpm-store/
.pnpm-debug.log
npm-debug.log*
yarn-debug.log*
yarn-error.log*

# Build artifacts
dist/
build/
*.tsbuildinfo
next-env.d.ts

# Generated TS bindings (regenerated from Rust)
lib/bindings.ts

# Rust / Tauri
src-tauri/target/
src-tauri/Cargo.lock
**/*.rs.bk

# Editor / OS
.DS_Store
Thumbs.db
.idea/
.vscode/*
!.vscode/settings.json
!.vscode/extensions.json

# Env
.env
.env.local
.env.*.local

# Tauri build output
src-tauri/gen/
```

- [x] **Step 4: Stage and commit**

```bash
git add .gitignore
git commit -m "chore: initialize repo and add .gitignore"
```

Expected: Commit succeeds with one file changed.

---

## Task 2: Create `package.json`

**Files:**
- Create: `package.json`

- [ ] **Step 1: Write `package.json`**

```json
{
  "name": "ai-software-studio",
  "version": "0.0.1",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start",
    "lint": "next lint",
    "typecheck": "tsc --noEmit",
    "tauri": "tauri",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build",
    "gen:bindings": "cargo test --manifest-path src-tauri/Cargo.toml export_bindings -- --nocapture"
  },
  "dependencies": {},
  "devDependencies": {},
  "packageManager": "pnpm@9.12.0"
}
```

- [ ] **Step 2: Verify pnpm is installed**

Run: `pnpm --version`
Expected: Prints a version ≥ 9.0.0. If pnpm is missing, install via `corepack enable && corepack prepare pnpm@9.12.0 --activate`.

- [ ] **Step 3: Commit**

```bash
git add package.json
git commit -m "chore: add package.json with pnpm scripts"
```

---

## Task 3: Install Next.js 15, React 19, TypeScript

**Files:**
- Modify: `package.json` (via pnpm add)

- [ ] **Step 1: Install runtime dependencies**

```bash
pnpm add next@15 react@19 react-dom@19
```

Expected: `package.json` now lists `next`, `react`, `react-dom` under `dependencies`; `node_modules/` and `pnpm-lock.yaml` are created.

- [ ] **Step 2: Install TypeScript and types**

```bash
pnpm add -D typescript @types/node @types/react @types/react-dom
```

- [ ] **Step 3: Install Zustand and TanStack Query**

```bash
pnpm add zustand @tanstack/react-query
```

- [ ] **Step 4: Install next-themes and lucide-react**

```bash
pnpm add next-themes lucide-react
```

- [ ] **Step 5: Commit**

```bash
git add package.json pnpm-lock.yaml
git commit -m "chore: install Next.js 15, React 19, Zustand, TanStack Query, next-themes"
```

---

## Task 4: Write `tsconfig.json` and `next.config.ts`

**Files:**
- Create: `tsconfig.json`
- Create: `next.config.ts`

- [ ] **Step 1: Write `tsconfig.json`**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["dom", "dom.iterable", "esnext"],
    "allowJs": false,
    "skipLibCheck": true,
    "strict": true,
    "noEmit": true,
    "esModuleInterop": true,
    "module": "esnext",
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "jsx": "preserve",
    "incremental": true,
    "plugins": [{ "name": "next" }],
    "paths": {
      "@/*": ["./*"]
    }
  },
  "include": ["next-env.d.ts", "**/*.ts", "**/*.tsx", ".next/types/**/*.ts"],
  "exclude": ["node_modules", "src-tauri", "out", ".next"]
}
```

- [ ] **Step 2: Write `next.config.ts`**

```ts
import type { NextConfig } from "next";

const config: NextConfig = {
  output: "export",
  images: { unoptimized: true },
  trailingSlash: true,
  reactStrictMode: true,
};

export default config;
```

- [ ] **Step 3: Commit**

```bash
git add tsconfig.json next.config.ts
git commit -m "chore: add TypeScript and Next.js config for static export"
```

---

## Task 5: ESLint configuration

**Files:**
- Create: `eslint.config.mjs`
- Modify: `package.json` (via pnpm add)

- [ ] **Step 1: Install ESLint and Next plugin**

```bash
pnpm add -D eslint eslint-config-next@15
```

- [ ] **Step 2: Write `eslint.config.mjs`** (flat config — Next 15 default)

```js
import { FlatCompat } from "@eslint/eslintrc";

const compat = new FlatCompat({ baseDirectory: import.meta.dirname });

const config = [
  ...compat.extends("next/core-web-vitals", "next/typescript"),
  {
    ignores: ["src-tauri/**", "out/**", ".next/**", "node_modules/**", "lib/bindings.ts"],
  },
];

export default config;
```

- [ ] **Step 3: Install the flat-config compat package**

```bash
pnpm add -D @eslint/eslintrc
```

- [ ] **Step 4: Run lint (will pass — no source code yet)**

Run: `pnpm lint`
Expected: ESLint runs against zero TS files and exits 0 (or prompts to configure if needed — answer with `Strict (recommended)` if asked).

- [ ] **Step 5: Commit**

```bash
git add eslint.config.mjs package.json pnpm-lock.yaml
git commit -m "chore: configure ESLint with Next.js flat config"
```

---

## Task 6: Verify baseline tooling works

**Files:** none

- [ ] **Step 1: Run typecheck**

Run: `pnpm typecheck`
Expected: tsc exits 0 (no files to typecheck yet, but the config is valid).

- [ ] **Step 2: Confirm git status is clean**

Run: `git status`
Expected: "nothing to commit, working tree clean".

---

## Task 7: Install Tailwind v4 and configure PostCSS

**Files:**
- Create: `postcss.config.mjs`

- [ ] **Step 1: Install Tailwind v4 and its PostCSS plugin**

```bash
pnpm add -D tailwindcss@4 @tailwindcss/postcss
```

- [ ] **Step 2: Write `postcss.config.mjs`**

```js
const config = {
  plugins: {
    "@tailwindcss/postcss": {},
  },
};

export default config;
```

- [ ] **Step 3: Commit**

```bash
git add postcss.config.mjs package.json pnpm-lock.yaml
git commit -m "chore: install Tailwind v4 and PostCSS config"
```

---

## Task 8: Write `app/globals.css` with two-layer token system

**Files:**
- Create: `app/globals.css`

- [ ] **Step 1: Create the `app/` directory and write `globals.css`**

```bash
mkdir -p app
```

Then create `app/globals.css` with exactly this content (matches the spec §6.2 token layout):

```css
@import "tailwindcss";

@custom-variant dark (&:where(.dark, .dark *));

/* ----- Semantic tokens (read by shadcn-generated CSS) ----- */
/* :root holds the LIGHT palette per shadcn convention */
:root {
  --background: oklch(0.98 0.005 250);
  --foreground: oklch(0.20 0.02 265);
  --card: oklch(1 0 0);
  --card-foreground: oklch(0.20 0.02 265);
  --popover: oklch(1 0 0);
  --popover-foreground: oklch(0.20 0.02 265);
  --primary: oklch(0.55 0.20 290);
  --primary-foreground: oklch(0.98 0 0);
  --secondary: oklch(0.62 0.14 200);
  --secondary-foreground: oklch(0.10 0.02 265);
  --muted: oklch(0.96 0.005 250);
  --muted-foreground: oklch(0.50 0.02 260);
  --accent: oklch(0.65 0.16 330);
  --accent-foreground: oklch(0.10 0.02 265);
  --destructive: oklch(0.55 0.22 25);
  --destructive-foreground: oklch(0.98 0 0);
  --success: oklch(0.62 0.18 145);
  --warning: oklch(0.70 0.16 75);
  --border: oklch(0.90 0.01 250);
  --input: oklch(0.92 0.01 250);
  --ring: oklch(0.55 0.20 290);
  --radius: 0.875rem;
}

/* .dark overrides — what `<html class="dark">` triggers */
.dark {
  --background: oklch(0.18 0.025 265);
  --foreground: oklch(0.95 0.01 250);
  --card: oklch(0.22 0.03 265);
  --card-foreground: oklch(0.95 0.01 250);
  --popover: oklch(0.22 0.03 265);
  --popover-foreground: oklch(0.95 0.01 250);
  --primary: oklch(0.65 0.20 290);
  --primary-foreground: oklch(0.98 0 0);
  --secondary: oklch(0.78 0.14 200);
  --secondary-foreground: oklch(0.10 0.02 265);
  --muted: oklch(0.26 0.025 265);
  --muted-foreground: oklch(0.70 0.02 260);
  --accent: oklch(0.75 0.16 330);
  --accent-foreground: oklch(0.10 0.02 265);
  --destructive: oklch(0.65 0.22 25);
  --destructive-foreground: oklch(0.98 0 0);
  --success: oklch(0.78 0.18 145);
  --warning: oklch(0.80 0.16 75);
  --border: oklch(0.32 0.03 265 / 0.6);
  --input: oklch(0.32 0.03 265);
  --ring: oklch(0.65 0.20 290);
}

/* ----- Tailwind utility mapping ----- */
@theme inline {
  --color-background: var(--background);
  --color-foreground: var(--foreground);
  --color-card: var(--card);
  --color-card-foreground: var(--card-foreground);
  --color-popover: var(--popover);
  --color-popover-foreground: var(--popover-foreground);
  --color-primary: var(--primary);
  --color-primary-foreground: var(--primary-foreground);
  --color-secondary: var(--secondary);
  --color-secondary-foreground: var(--secondary-foreground);
  --color-muted: var(--muted);
  --color-muted-foreground: var(--muted-foreground);
  --color-accent: var(--accent);
  --color-accent-foreground: var(--accent-foreground);
  --color-destructive: var(--destructive);
  --color-destructive-foreground: var(--destructive-foreground);
  --color-success: var(--success);
  --color-warning: var(--warning);
  --color-border: var(--border);
  --color-input: var(--input);
  --color-ring: var(--ring);
  --radius-lg: var(--radius);
  --font-sans: var(--font-inter), ui-sans-serif, system-ui, sans-serif;
  --font-mono: var(--font-jetbrains-mono), ui-monospace, monospace;
}

/* ----- Panel surface (used by PanelFrame) ----- */
.panel-surface {
  background: linear-gradient(
    180deg,
    color-mix(in oklch, var(--card) 92%, white 8%) 0%,
    var(--card) 100%
  );
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: 0 1px 0 0 oklch(1 0 0 / 0.04) inset;
}

body {
  background: var(--background);
  color: var(--foreground);
  font-family: var(--font-sans);
}
```

- [ ] **Step 2: Commit**

```bash
git add app/globals.css
git commit -m "feat(theme): add Tailwind v4 globals with shadcn two-layer tokens"
```

---

## Task 9: Initialize shadcn config (`components.json`)

**Files:**
- Create: `components.json`

- [ ] **Step 1: Write `components.json`**

```json
{
  "$schema": "https://ui.shadcn.com/schema.json",
  "style": "new-york",
  "rsc": false,
  "tsx": true,
  "tailwind": {
    "config": "",
    "css": "app/globals.css",
    "baseColor": "neutral",
    "cssVariables": true,
    "prefix": ""
  },
  "aliases": {
    "components": "@/components",
    "utils": "@/lib/utils",
    "ui": "@/components/ui",
    "lib": "@/lib",
    "hooks": "@/hooks"
  },
  "iconLibrary": "lucide"
}
```

- [ ] **Step 2: Commit**

```bash
git add components.json
git commit -m "chore: add shadcn components.json (new-york, Tailwind v4)"
```

---

## Task 10: Install first-pass shadcn components

**Files:**
- Create: `components/ui/{button,card,badge,separator,scroll-area,tooltip,avatar,checkbox,tabs,dropdown-menu}.tsx`
- Create: `lib/utils.ts` (auto-created by shadcn)

- [ ] **Step 1: Run shadcn CLI for the batch**

```bash
pnpm dlx shadcn@latest add button card badge separator scroll-area tooltip avatar checkbox tabs dropdown-menu
```

If the CLI prompts for missing files (e.g., "create lib/utils.ts?"), accept all defaults. If it prompts about React 19 peer-dep, answer `--legacy-peer-deps` style "use --force" — the components are stable on React 19.

Expected: Ten files written under `components/ui/`, one at `lib/utils.ts`, and `class-variance-authority`, `clsx`, `tailwind-merge` added to dependencies.

- [ ] **Step 2: Inspect `lib/utils.ts`**

Run: `cat lib/utils.ts`
Expected output:

```ts
import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
```

If shadcn didn't create `lib/utils.ts`, create it manually with the content above.

- [ ] **Step 3: Commit**

```bash
git add components/ui lib/utils.ts package.json pnpm-lock.yaml components.json
git commit -m "feat(ui): install shadcn primitives (button, card, badge, etc.)"
```

---

## Task 11: Local fonts setup + initial providers file

**Files:**
- Create: `public/fonts/.gitkeep`
- Create: `app/providers.tsx`

- [ ] **Step 1: Create the fonts directory and a placeholder note**

```bash
mkdir -p public/fonts
```

Write `public/fonts/README.md`:

```markdown
# Local fonts

Place these files here:

- `Inter.woff2` — Inter Variable (download from https://rsms.me/inter/ or fontsource)
- `JetBrainsMono.woff2` — JetBrains Mono Variable (download from fontsource)

Both are referenced from `app/layout.tsx` via `next/font/local`. Builds will fail if these files are missing — that's intentional. Local fonts keep `pnpm build` and `pnpm tauri build` working without network access.
```

Add a `.gitkeep` so the directory survives an empty checkout:

```bash
touch public/fonts/.gitkeep
```

- [ ] **Step 2: Download both font files**

```bash
curl -fsSL -o public/fonts/Inter.woff2 https://cdn.jsdelivr.net/fontsource/fonts/inter:vf@latest/latin-wght-normal.woff2
curl -fsSL -o public/fonts/JetBrainsMono.woff2 https://cdn.jsdelivr.net/fontsource/fonts/jetbrains-mono:vf@latest/latin-wght-normal.woff2
ls -la public/fonts/
```

Expected: Both files exist and are > 50 KB. If the download fails (offline), fall back to manually downloading from https://fontsource.org/fonts/inter and https://fontsource.org/fonts/jetbrains-mono — the `*.woff2` variable files.

- [ ] **Step 3: Write `app/providers.tsx`** (theme provider only; TanStack Query added in Task 34)

```tsx
"use client";

import { ThemeProvider } from "next-themes";
import type { ReactNode } from "react";

export function Providers({ children }: { children: ReactNode }) {
  return (
    <ThemeProvider
      attribute="class"
      defaultTheme="dark"
      enableSystem
      disableTransitionOnChange
    >
      {children}
    </ThemeProvider>
  );
}
```

- [ ] **Step 4: Commit**

```bash
git add public/fonts app/providers.tsx
git commit -m "feat(theme): add local fonts and next-themes provider"
```

---

## Task 12: Build `ThemeToggle` component

**Files:**
- Create: `components/layout/theme-toggle.tsx`
- Create: `hooks/use-mounted.ts`

- [ ] **Step 1: Write `hooks/use-mounted.ts`**

```bash
mkdir -p hooks
```

```ts
"use client";

import { useEffect, useState } from "react";

export function useMounted() {
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);
  return mounted;
}
```

- [ ] **Step 2: Write `components/layout/theme-toggle.tsx`**

```bash
mkdir -p components/layout
```

```tsx
"use client";

import { Moon, Sun } from "lucide-react";
import { useTheme } from "next-themes";
import { Button } from "@/components/ui/button";
import { useMounted } from "@/hooks/use-mounted";

export function ThemeToggle() {
  const { resolvedTheme, setTheme } = useTheme();
  const mounted = useMounted();

  if (!mounted) {
    return (
      <Button variant="ghost" size="icon" aria-label="Toggle theme">
        <span className="h-4 w-4" />
      </Button>
    );
  }

  const isDark = resolvedTheme === "dark";

  return (
    <Button
      variant="ghost"
      size="icon"
      aria-label={isDark ? "Switch to light theme" : "Switch to dark theme"}
      onClick={() => setTheme(isDark ? "light" : "dark")}
    >
      {isDark ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
    </Button>
  );
}
```

- [ ] **Step 3: Commit**

```bash
git add hooks components/layout/theme-toggle.tsx
git commit -m "feat(theme): add ThemeToggle component"
```

---

## Task 13: Root layout + initial placeholder page

**Files:**
- Create: `app/layout.tsx`
- Create: `app/page.tsx`

- [ ] **Step 1: Write `app/layout.tsx`**

```tsx
import type { Metadata } from "next";
import localFont from "next/font/local";
import { Providers } from "./providers";
import "./globals.css";

const inter = localFont({
  src: "../public/fonts/Inter.woff2",
  variable: "--font-inter",
  display: "swap",
  weight: "100 900",
});

const jetbrainsMono = localFont({
  src: "../public/fonts/JetBrainsMono.woff2",
  variable: "--font-jetbrains-mono",
  display: "swap",
  weight: "100 800",
});

export const metadata: Metadata = {
  title: "AI Software Studio",
  description: "The AI-first IDE — watch first, evidence-first, human accountable.",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" suppressHydrationWarning className={`${inter.variable} ${jetbrainsMono.variable}`}>
      <body>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
```

- [ ] **Step 2: Write placeholder `app/page.tsx`** (rewritten in Task 40 once panels exist)

```tsx
import { ThemeToggle } from "@/components/layout/theme-toggle";

export default function HomePage() {
  return (
    <main className="flex min-h-screen items-center justify-center p-8">
      <div className="flex flex-col items-center gap-4">
        <h1 className="text-3xl font-semibold">AI Software Studio</h1>
        <p className="text-muted-foreground">Scaffold checkpoint — panels coming next.</p>
        <ThemeToggle />
      </div>
    </main>
  );
}
```

- [ ] **Step 3: Run dev server and visually verify**

Run: `pnpm dev`
In a browser, open `http://localhost:3000`.
Expected: Dark page with "AI Software Studio" title and a working sun/moon toggle. Press the toggle — page should swap to light mode.

- [ ] **Step 4: Stop the dev server**

Hit Ctrl+C in the terminal.

- [ ] **Step 5: Run typecheck and build**

```bash
pnpm typecheck && pnpm build
```

Expected: Both succeed. `out/` directory now exists with static HTML.

- [ ] **Step 6: Commit**

```bash
git add app/layout.tsx app/page.tsx
git commit -m "feat(app): root layout with local fonts and placeholder page"
```

---

## Task 14: Initialize Tauri (`src-tauri/`) — Cargo.toml + tauri.conf.json + build.rs

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Verify Rust toolchain**

Run: `rustc --version && cargo --version`
Expected: Both print versions (Rust ≥ 1.78). If missing, install via `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` and re-open the shell.

- [ ] **Step 2: Create directory and write `Cargo.toml`**

```bash
mkdir -p src-tauri/src src-tauri/capabilities src-tauri/icons
```

`src-tauri/Cargo.toml`:

```toml
[package]
name = "ai-software-studio"
version = "0.0.1"
description = "AI Software Studio — local-first AI coding agent command center"
edition = "2021"
rust-version = "1.78"

[lib]
name = "ai_software_studio_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
tokio = { version = "1", features = ["sync"] }
specta = { version = "2.0.0-rc.20", features = ["derive"] }
tauri-specta = { version = "2.0.0-rc.20", features = ["derive", "typescript"] }
specta-typescript = "0.0.7"

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

- [ ] **Step 3: Write `src-tauri/build.rs`**

```rust
fn main() {
    tauri_build::build()
}
```

- [ ] **Step 4: Write `src-tauri/tauri.conf.json`**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "AI Software Studio",
  "version": "0.0.1",
  "identifier": "studio.aisoftware.app",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build",
    "devUrl": "http://localhost:3000",
    "frontendDist": "../out"
  },
  "app": {
    "windows": [
      {
        "title": "AI Software Studio",
        "width": 1440,
        "height": 900,
        "minWidth": 1200,
        "minHeight": 800,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

- [ ] **Step 5: Add Tauri JS API**

```bash
pnpm add @tauri-apps/api@2
pnpm add -D @tauri-apps/cli@2
```

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/build.rs src-tauri/tauri.conf.json package.json pnpm-lock.yaml
git commit -m "chore(tauri): scaffold Cargo.toml, build.rs, tauri.conf.json"
```

---

## Task 15: Capabilities and placeholder icons

**Files:**
- Create: `src-tauri/capabilities/default.json`
- Create: `src-tauri/icons/*` (placeholder)

- [ ] **Step 1: Write capabilities**

`src-tauri/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capabilities for AI Software Studio",
  "windows": ["main"],
  "permissions": ["core:default"]
}
```

- [ ] **Step 2: Generate placeholder icons**

The Tauri CLI ships with an icon generator that takes a single source PNG and produces all required sizes. Use a temporary 1024×1024 solid-color PNG so the build works; we'll replace it with a real icon later.

```bash
pnpm dlx @tauri-apps/cli@2 icon --help >/dev/null 2>&1 || true
python3 - <<'PY'
import struct, zlib, pathlib
size = 1024
# Solid indigo PNG (#5B4FE6)
header = b'\x89PNG\r\n\x1a\n'
def chunk(t, d):
    return struct.pack('>I', len(d)) + t + d + struct.pack('>I', zlib.crc32(t + d) & 0xffffffff)
ihdr = chunk(b'IHDR', struct.pack('>IIBBBBB', size, size, 8, 2, 0, 0, 0))
raw = b''
for _ in range(size):
    raw += b'\x00' + (b'\x5b\x4f\xe6' * size)
idat = chunk(b'IDAT', zlib.compress(raw, 9))
iend = chunk(b'IEND', b'')
pathlib.Path('src-tauri/icons').mkdir(parents=True, exist_ok=True)
pathlib.Path('src-tauri/icons/_seed.png').write_bytes(header + ihdr + idat + iend)
PY
pnpm dlx @tauri-apps/cli@2 icon src-tauri/icons/_seed.png -o src-tauri/icons
ls src-tauri/icons
```

Expected: `src-tauri/icons/` contains `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.icns`, `icon.ico`, plus other generated sizes. If the icon generator fails on Linux (missing ImageMagick/iconutil dependencies), document the failure and supply icons manually for the developer's platform.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/capabilities src-tauri/icons
git commit -m "chore(tauri): add default capabilities and placeholder icons"
```

---

## Task 16: First Rust build check (empty crate)

**Files:**
- Create: `src-tauri/src/main.rs` (temporary stub)
- Create: `src-tauri/src/lib.rs` (temporary stub)

- [ ] **Step 1: Write a minimal stub so Cargo can build**

`src-tauri/src/lib.rs`:

```rust
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`src-tauri/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ai_software_studio_lib::run()
}
```

- [ ] **Step 2: Build**

```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

Expected: Cargo downloads crates and produces a debug binary in `src-tauri/target/debug/`. This takes 2–5 minutes the first time.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src
git commit -m "chore(tauri): minimal Rust stub that builds"
```

---

## Task 17: Rust — `error.rs` (AppError type)

**Files:**
- Create: `src-tauri/src/error.rs`

- [ ] **Step 1: Write `src-tauri/src/error.rs`**

```rust
use serde::{Serialize, Deserialize};
use specta::Type;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum AppErrorCode {
    NotFound,
    InvalidArg,
    Internal,
    EngineNotReady,
    Unimplemented,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Error)]
#[serde(rename_all = "camelCase")]
#[error("{code:?}: {message}")]
pub struct AppError {
    pub code: AppErrorCode,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl AppError {
    pub fn not_found(message: impl Into<String>) -> Self {
        Self { code: AppErrorCode::NotFound, message: message.into(), details: None }
    }

    pub fn invalid_arg(message: impl Into<String>) -> Self {
        Self { code: AppErrorCode::InvalidArg, message: message.into(), details: None }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self { code: AppErrorCode::Internal, message: message.into(), details: None }
    }

    pub fn unimplemented(message: impl Into<String>) -> Self {
        Self { code: AppErrorCode::Unimplemented, message: message.into(), details: None }
    }
}
```

- [ ] **Step 2: Build to confirm it compiles** (we need to register it in `lib.rs` first — the next task wires it up)

We won't build until Task 22 hooks `error` and `models` into `lib.rs`.

---

## Task 18: Rust — `models.rs`

**Files:**
- Create: `src-tauri/src/models.rs`

- [ ] **Step 1: Write `src-tauri/src/models.rs`**

```rust
use serde::{Serialize, Deserialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub default_branch: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TaskStatus {
    Draft,
    WorktreeCreated,
    Running,
    NeedsInput,
    VerificationRunning,
    ReviewReady,
    Approved,
    PrPrepared,
    Done,
    ChangesRequested,
    Rejected,
    Failed,
    Stopped,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RiskLevel { Safe, Sensitive, Dependency, Migration, Infra, Unknown }

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub constraints: Vec<String>,
    pub selected_engine: Option<String>,
    pub status: TaskStatus,
    pub risk: RiskLevel,
    pub branch_name: Option<String>,
    pub worktree_path: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AcceptanceCriterion {
    pub id: String,
    pub label: String,
    pub satisfied: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EngineDetectionStatus { NotInstalled, Detected, Ready, NotAuthenticated, Error }

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub status: EngineDetectionStatus,
    pub binary_path: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum VerificationStatus { NotRun, Running, Passed, Failed, Skipped, Warning }

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VerificationCheck {
    pub kind: String,
    pub status: VerificationStatus,
    pub duration_ms: Option<u64>,
    pub log_excerpt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VerificationRun {
    pub id: String,
    pub task_id: String,
    pub started_at: String,
    pub checks: Vec<VerificationCheck>,
}
```

---

## Task 19: Rust — `fixtures.rs`

**Files:**
- Create: `src-tauri/src/fixtures.rs`

- [ ] **Step 1: Write `src-tauri/src/fixtures.rs`**

```rust
// Keep in sync with lib/mock-data.ts. lib/mock-data.ts is the canonical
// shape; if you change something here, mirror it there.

use crate::models::*;

pub fn projects() -> Vec<Project> {
    vec![Project {
        id: "proj-default".into(),
        name: "example-app".into(),
        path: "/Users/dev/example-app".into(),
        default_branch: "main".into(),
    }]
}

pub fn tasks_for_project(project_id: &str) -> Vec<Task> {
    if project_id != "proj-default" {
        return vec![];
    }
    vec![
        Task {
            id: "task-042".into(),
            project_id: project_id.into(),
            title: "Add magic link login while preserving JWT flow".into(),
            description: "Migrate sign-in to email magic links without breaking the existing JWT-based session handler.".into(),
            acceptance_criteria: vec![
                AcceptanceCriterion { id: "ac1".into(), label: "Magic link email delivered in dev".into(), satisfied: true },
                AcceptanceCriterion { id: "ac2".into(), label: "Existing JWT routes still pass".into(), satisfied: true },
                AcceptanceCriterion { id: "ac3".into(), label: "Session cookie behavior unchanged".into(), satisfied: false },
            ],
            constraints: vec!["No new external dependencies".into(), "Do not modify src/billing".into()],
            selected_engine: Some("claude-code".into()),
            status: TaskStatus::ReviewReady,
            risk: RiskLevel::Sensitive,
            branch_name: Some("aistudio/task-42-magic-link".into()),
            worktree_path: Some("/Users/dev/.aistudio/worktrees/example-app/task-042".into()),
            created_at: "2026-05-15T10:00:00Z".into(),
        },
        Task {
            id: "task-041".into(),
            project_id: project_id.into(),
            title: "Fix race in checkout cancellation".into(),
            description: "Investigate intermittent failure when a user cancels checkout mid-payment.".into(),
            acceptance_criteria: vec![
                AcceptanceCriterion { id: "ac1".into(), label: "Reproducer test added".into(), satisfied: false },
                AcceptanceCriterion { id: "ac2".into(), label: "No regressions in /checkout".into(), satisfied: false },
            ],
            constraints: vec!["Run full test suite".into()],
            selected_engine: Some("codex-cli".into()),
            status: TaskStatus::Running,
            risk: RiskLevel::Safe,
            branch_name: Some("aistudio/task-41-checkout-race".into()),
            worktree_path: Some("/Users/dev/.aistudio/worktrees/example-app/task-041".into()),
            created_at: "2026-05-16T14:00:00Z".into(),
        },
        Task {
            id: "task-040".into(),
            project_id: project_id.into(),
            title: "Reduce dashboard query latency".into(),
            description: "P95 is 1.2s; target 400ms.".into(),
            acceptance_criteria: vec![
                AcceptanceCriterion { id: "ac1".into(), label: "P95 under 400ms in load test".into(), satisfied: false },
            ],
            constraints: vec![],
            selected_engine: None,
            status: TaskStatus::Draft,
            risk: RiskLevel::Safe,
            branch_name: None,
            worktree_path: None,
            created_at: "2026-05-17T09:00:00Z".into(),
        },
        Task {
            id: "task-039".into(),
            project_id: project_id.into(),
            title: "Improve onboarding empty state".into(),
            description: "Show users a guided path on first login.".into(),
            acceptance_criteria: vec![],
            constraints: vec![],
            selected_engine: Some("claude-code".into()),
            status: TaskStatus::Approved,
            risk: RiskLevel::Safe,
            branch_name: Some("aistudio/task-39-onboarding".into()),
            worktree_path: Some("/Users/dev/.aistudio/worktrees/example-app/task-039".into()),
            created_at: "2026-05-14T11:00:00Z".into(),
        },
        Task {
            id: "task-038".into(),
            project_id: project_id.into(),
            title: "Refactor billing webhook handler".into(),
            description: "Split the 600-line handler into intent-scoped sub-handlers.".into(),
            acceptance_criteria: vec![],
            constraints: vec!["Do not change webhook public contract".into()],
            selected_engine: None,
            status: TaskStatus::ChangesRequested,
            risk: RiskLevel::Sensitive,
            branch_name: Some("aistudio/task-38-webhook-refactor".into()),
            worktree_path: None,
            created_at: "2026-05-13T15:00:00Z".into(),
        },
    ]
}

pub fn engines() -> Vec<EngineStatus> {
    vec![
        EngineStatus {
            id: "claude-code".into(),
            name: "Claude Code".into(),
            version: Some("0.43.1".into()),
            status: EngineDetectionStatus::Ready,
            binary_path: Some("/opt/homebrew/bin/claude".into()),
        },
        EngineStatus {
            id: "codex-cli".into(),
            name: "Codex CLI".into(),
            version: Some("0.125.0".into()),
            status: EngineDetectionStatus::NotAuthenticated,
            binary_path: Some("/opt/homebrew/bin/codex".into()),
        },
    ]
}

pub fn verification_for_task(task_id: &str) -> Vec<VerificationRun> {
    if task_id != "task-042" {
        return vec![];
    }
    vec![VerificationRun {
        id: "vr-001".into(),
        task_id: task_id.into(),
        started_at: "2026-05-17T12:00:00Z".into(),
        checks: vec![
            VerificationCheck { kind: "install".into(), status: VerificationStatus::Passed, duration_ms: Some(8400), log_excerpt: Some("Lockfile up to date".into()) },
            VerificationCheck { kind: "typecheck".into(), status: VerificationStatus::Passed, duration_ms: Some(3200), log_excerpt: None },
            VerificationCheck { kind: "lint".into(), status: VerificationStatus::Warning, duration_ms: Some(1100), log_excerpt: Some("2 warnings: unused import in auth.ts".into()) },
            VerificationCheck { kind: "test".into(), status: VerificationStatus::Passed, duration_ms: Some(18000), log_excerpt: Some("142 passed, 0 failed".into()) },
            VerificationCheck { kind: "build".into(), status: VerificationStatus::Failed, duration_ms: Some(22000), log_excerpt: Some("Type error in middleware.ts:88".into()) },
        ],
    }]
}
```

---

## Task 20: Rust — Service modules (mock-backed)

**Files:**
- Create: `src-tauri/src/tasks/mod.rs`
- Create: `src-tauri/src/projects/mod.rs`
- Create: `src-tauri/src/engines/mod.rs`
- Create: `src-tauri/src/verification/mod.rs`

- [ ] **Step 1: Write `src-tauri/src/tasks/mod.rs`**

```bash
mkdir -p src-tauri/src/tasks src-tauri/src/projects src-tauri/src/engines src-tauri/src/verification
```

```rust
use crate::{error::AppError, fixtures, models::Task};

pub struct TaskService;

impl TaskService {
    pub fn new() -> Self { Self }

    pub async fn list_for_project(&self, project_id: &str) -> Result<Vec<Task>, AppError> {
        Ok(fixtures::tasks_for_project(project_id))
    }

    pub async fn get(&self, task_id: &str) -> Result<Task, AppError> {
        fixtures::tasks_for_project("proj-default")
            .into_iter()
            .find(|t| t.id == task_id)
            .ok_or_else(|| AppError::not_found(format!("task {task_id} not found")))
    }
}
```

- [ ] **Step 2: Write `src-tauri/src/projects/mod.rs`**

```rust
use crate::{error::AppError, fixtures, models::Project};

pub struct ProjectService;

impl ProjectService {
    pub fn new() -> Self { Self }

    pub async fn list(&self) -> Result<Vec<Project>, AppError> {
        Ok(fixtures::projects())
    }
}
```

- [ ] **Step 3: Write `src-tauri/src/engines/mod.rs`**

```rust
use crate::{error::AppError, fixtures, models::EngineStatus};

pub struct EngineService;

impl EngineService {
    pub fn new() -> Self { Self }

    pub async fn list(&self) -> Result<Vec<EngineStatus>, AppError> {
        Ok(fixtures::engines())
    }

    /// Phase 1 stub. Replaced with real `which`/`--version` shelling in a later phase.
    pub async fn detect(&self) -> Result<Vec<EngineStatus>, AppError> {
        Ok(fixtures::engines())
    }
}
```

- [ ] **Step 4: Write `src-tauri/src/verification/mod.rs`**

```rust
use crate::{error::AppError, fixtures, models::VerificationRun};

pub struct VerificationService;

impl VerificationService {
    pub fn new() -> Self { Self }

    pub async fn list_for_task(&self, task_id: &str) -> Result<Vec<VerificationRun>, AppError> {
        Ok(fixtures::verification_for_task(task_id))
    }
}
```

---

## Task 21: Rust — Placeholder modules

**Files:**
- Create: `src-tauri/src/{core,db,git,process,policy,artifacts,config}/mod.rs`

- [ ] **Step 1: Create each placeholder module**

```bash
for m in core db git process policy artifacts config; do
  mkdir -p "src-tauri/src/$m"
  cat > "src-tauri/src/$m/mod.rs" <<EOF
//! ${m} module — placeholder for a later phase.
//!
//! See docs/architecture/architecture.md §8.2 and §16 for the intended
//! responsibilities. This module exists now so commands and services can
//! depend on it as features land.

// TODO: implement in Phase ${m} work.
EOF
done
ls src-tauri/src
```

Expected: Each of `core/`, `db/`, `git/`, `process/`, `policy/`, `artifacts/`, `config/` exists with a `mod.rs`.

---

## Task 22: Rust — `state.rs` (AppState)

**Files:**
- Create: `src-tauri/src/state.rs`

- [ ] **Step 1: Write `src-tauri/src/state.rs`**

```rust
use crate::{
    engines::EngineService,
    projects::ProjectService,
    tasks::TaskService,
    verification::VerificationService,
};

pub struct AppState {
    pub tasks: TaskService,
    pub projects: ProjectService,
    pub engines: EngineService,
    pub verification: VerificationService,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            tasks: TaskService::new(),
            projects: ProjectService::new(),
            engines: EngineService::new(),
            verification: VerificationService::new(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self { Self::new() }
}
```

---

## Task 23: Rust — Command modules (thin wrappers)

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/{projects,tasks,engines,verification}.rs`

- [ ] **Step 1: Write `src-tauri/src/commands/mod.rs`**

```bash
mkdir -p src-tauri/src/commands
```

```rust
pub mod engines;
pub mod projects;
pub mod tasks;
pub mod verification;
```

- [ ] **Step 2: Write `src-tauri/src/commands/projects.rs`**

```rust
use crate::{error::AppError, models::Project, state::AppState};
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, AppError> {
    state.projects.list().await
}
```

- [ ] **Step 3: Write `src-tauri/src/commands/tasks.rs`**

```rust
use crate::{error::AppError, models::Task, state::AppState};
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn list_tasks(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<Task>, AppError> {
    state.tasks.list_for_project(&project_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Task, AppError> {
    state.tasks.get(&task_id).await
}
```

- [ ] **Step 4: Write `src-tauri/src/commands/engines.rs`**

```rust
use crate::{error::AppError, models::EngineStatus, state::AppState};
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn list_engines(state: State<'_, AppState>) -> Result<Vec<EngineStatus>, AppError> {
    state.engines.list().await
}

#[tauri::command]
#[specta::specta]
pub async fn detect_engines(state: State<'_, AppState>) -> Result<Vec<EngineStatus>, AppError> {
    state.engines.detect().await
}
```

- [ ] **Step 5: Write `src-tauri/src/commands/verification.rs`**

```rust
use crate::{error::AppError, models::VerificationRun, state::AppState};
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn list_verification(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Vec<VerificationRun>, AppError> {
    state.verification.list_for_task(&task_id).await
}
```

---

## Task 24: Rust — `main.rs` + `lib.rs` (wiring + specta builder)

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Rewrite `src-tauri/src/lib.rs`**

```rust
pub mod artifacts;
pub mod commands;
pub mod config;
pub mod core;
pub mod db;
pub mod engines;
pub mod error;
pub mod fixtures;
pub mod git;
pub mod models;
pub mod policy;
pub mod process;
pub mod projects;
pub mod state;
pub mod tasks;
pub mod verification;

use state::AppState;
use tauri_specta::{collect_commands, Builder};

pub fn run() {
    let specta_builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::projects::list_projects,
            commands::tasks::list_tasks,
            commands::tasks::get_task,
            commands::engines::list_engines,
            commands::engines::detect_engines,
            commands::verification::list_verification,
        ]);

    #[cfg(debug_assertions)]
    specta_builder
        .export(
            specta_typescript::Typescript::default().formatter(specta_typescript::formatter::prettier),
            "../lib/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            specta_builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod export_bindings_test {
    use super::*;

    #[test]
    fn export_bindings() {
        let builder = Builder::<tauri::Wry>::new()
            .commands(collect_commands![
                commands::projects::list_projects,
                commands::tasks::list_tasks,
                commands::tasks::get_task,
                commands::engines::list_engines,
                commands::engines::detect_engines,
                commands::verification::list_verification,
            ]);
        builder
            .export(specta_typescript::Typescript::default(), "../lib/bindings.ts")
            .expect("export bindings");
    }
}
```

- [ ] **Step 2: Confirm `src-tauri/src/main.rs` calls into `lib::run`** (it already does from Task 16). No change needed.

- [ ] **Step 3: Commit progress before the build attempt** (so we can roll back if specta versions don't align)

```bash
git add src-tauri/src
git commit -m "feat(rust): models, error, services, commands, AppState, specta wiring"
```

---

## Task 25: Generate `lib/bindings.ts` via `cargo test`

**Files:**
- Create (generated): `lib/bindings.ts`

- [ ] **Step 1: Run the export test**

```bash
cargo test --manifest-path src-tauri/Cargo.toml export_bindings -- --nocapture
```

Expected: Cargo compiles, the test runs, and `lib/bindings.ts` appears.

If the build fails because `tauri-specta` / `specta` / `specta-typescript` version constraints don't line up (this is the most common scaffold pain point — these crates are still RC and frequently rev'd), do the following:
1. Look at the error message to see which crate's version is rejected.
2. Run `cargo search specta` and `cargo search tauri-specta` to see the latest compatible versions.
3. Update `src-tauri/Cargo.toml` to matching versions.
4. Re-run `cargo test ... export_bindings`.
5. If still failing, ask the user for guidance — do not silently downgrade Tauri itself.

- [ ] **Step 2: Inspect the generated file**

Run: `head -40 lib/bindings.ts`
Expected: TypeScript exports like `export async function listProjects(): Promise<Project[]>` and interface definitions for `Project`, `Task`, `EngineStatus`, `VerificationRun`, `AppError`.

- [ ] **Step 3: Confirm the file is gitignored**

Run: `git check-ignore lib/bindings.ts && echo IGNORED || echo TRACKED`
Expected: `IGNORED` — matches `.gitignore` line `lib/bindings.ts`.

- [ ] **Step 4: Commit any Cargo.lock churn** (no source changes needed)

If `src-tauri/Cargo.toml` was edited to fix versions:

```bash
git add src-tauri/Cargo.toml
git commit -m "chore(rust): pin specta/tauri-specta versions that match"
```

Otherwise skip.

---

## Task 26: `lib/types.ts` — UI-only types

**Files:**
- Create: `lib/types.ts`

- [ ] **Step 1: Write `lib/types.ts`**

```ts
// UI-only types — NOT crossing the Tauri boundary.
// Boundary types live in lib/bindings.ts (generated from Rust).

export type ConversationAuthor = "user" | "agent" | "system";

export interface ConversationMessage {
  id: string;
  author: ConversationAuthor;
  authorName: string;
  body: string;
  timestamp: string;
}

export interface ContextGraphNode {
  id: string;
  label: string;
  kind: "task" | "engine" | "file" | "branch";
  x: number;
  y: number;
}

export interface ContextGraphEdge {
  from: string;
  to: string;
}

export interface SnapshotMetric {
  label: string;
  value: string;
  trend?: "up" | "down" | "flat";
}

export interface ActiveAgent {
  engineId: string;
  taskId: string;
  status: "running" | "idle" | "blocked";
}
```

---

## Task 27: `lib/mock-data.ts`

**Files:**
- Create: `lib/mock-data.ts`

- [ ] **Step 1: Write `lib/mock-data.ts` — must mirror `src-tauri/src/fixtures.rs`**

```ts
import type {
  Project,
  Task,
  EngineStatus,
  VerificationRun,
} from "./bindings";
import type {
  ConversationMessage,
  ContextGraphEdge,
  ContextGraphNode,
  SnapshotMetric,
  ActiveAgent,
} from "./types";

export const mockProjects: Project[] = [
  {
    id: "proj-default",
    name: "example-app",
    path: "/Users/dev/example-app",
    defaultBranch: "main",
  },
];

export const mockTasks: Task[] = [
  {
    id: "task-042",
    projectId: "proj-default",
    title: "Add magic link login while preserving JWT flow",
    description:
      "Migrate sign-in to email magic links without breaking the existing JWT-based session handler.",
    acceptanceCriteria: [
      { id: "ac1", label: "Magic link email delivered in dev", satisfied: true },
      { id: "ac2", label: "Existing JWT routes still pass", satisfied: true },
      { id: "ac3", label: "Session cookie behavior unchanged", satisfied: false },
    ],
    constraints: ["No new external dependencies", "Do not modify src/billing"],
    selectedEngine: "claude-code",
    status: "reviewReady",
    risk: "sensitive",
    branchName: "aistudio/task-42-magic-link",
    worktreePath: "/Users/dev/.aistudio/worktrees/example-app/task-042",
    createdAt: "2026-05-15T10:00:00Z",
  },
  {
    id: "task-041",
    projectId: "proj-default",
    title: "Fix race in checkout cancellation",
    description: "Investigate intermittent failure when a user cancels checkout mid-payment.",
    acceptanceCriteria: [
      { id: "ac1", label: "Reproducer test added", satisfied: false },
      { id: "ac2", label: "No regressions in /checkout", satisfied: false },
    ],
    constraints: ["Run full test suite"],
    selectedEngine: "codex-cli",
    status: "running",
    risk: "safe",
    branchName: "aistudio/task-41-checkout-race",
    worktreePath: "/Users/dev/.aistudio/worktrees/example-app/task-041",
    createdAt: "2026-05-16T14:00:00Z",
  },
  {
    id: "task-040",
    projectId: "proj-default",
    title: "Reduce dashboard query latency",
    description: "P95 is 1.2s; target 400ms.",
    acceptanceCriteria: [
      { id: "ac1", label: "P95 under 400ms in load test", satisfied: false },
    ],
    constraints: [],
    selectedEngine: null,
    status: "draft",
    risk: "safe",
    branchName: null,
    worktreePath: null,
    createdAt: "2026-05-17T09:00:00Z",
  },
  {
    id: "task-039",
    projectId: "proj-default",
    title: "Improve onboarding empty state",
    description: "Show users a guided path on first login.",
    acceptanceCriteria: [],
    constraints: [],
    selectedEngine: "claude-code",
    status: "approved",
    risk: "safe",
    branchName: "aistudio/task-39-onboarding",
    worktreePath: "/Users/dev/.aistudio/worktrees/example-app/task-039",
    createdAt: "2026-05-14T11:00:00Z",
  },
  {
    id: "task-038",
    projectId: "proj-default",
    title: "Refactor billing webhook handler",
    description: "Split the 600-line handler into intent-scoped sub-handlers.",
    acceptanceCriteria: [],
    constraints: ["Do not change webhook public contract"],
    selectedEngine: null,
    status: "changesRequested",
    risk: "sensitive",
    branchName: "aistudio/task-38-webhook-refactor",
    worktreePath: null,
    createdAt: "2026-05-13T15:00:00Z",
  },
];

export const mockEngines: EngineStatus[] = [
  {
    id: "claude-code",
    name: "Claude Code",
    version: "0.43.1",
    status: "ready",
    binaryPath: "/opt/homebrew/bin/claude",
  },
  {
    id: "codex-cli",
    name: "Codex CLI",
    version: "0.125.0",
    status: "notAuthenticated",
    binaryPath: "/opt/homebrew/bin/codex",
  },
];

export const mockVerification: VerificationRun[] = [
  {
    id: "vr-001",
    taskId: "task-042",
    startedAt: "2026-05-17T12:00:00Z",
    checks: [
      { kind: "install", status: "passed", durationMs: 8400, logExcerpt: "Lockfile up to date" },
      { kind: "typecheck", status: "passed", durationMs: 3200, logExcerpt: null },
      { kind: "lint", status: "warning", durationMs: 1100, logExcerpt: "2 warnings: unused import in auth.ts" },
      { kind: "test", status: "passed", durationMs: 18000, logExcerpt: "142 passed, 0 failed" },
      { kind: "build", status: "failed", durationMs: 22000, logExcerpt: "Type error in middleware.ts:88" },
    ],
  },
];

export const mockConversation: ConversationMessage[] = [
  { id: "m1", author: "user", authorName: "You", body: "Use magic link, keep JWT for now.", timestamp: "10:14" },
  { id: "m2", author: "agent", authorName: "Claude Code", body: "Acknowledged. Drafting changes in `src/auth/magic-link.ts`.", timestamp: "10:14" },
  { id: "m3", author: "agent", authorName: "Claude Code", body: "Added 14 lines, removed 3. Running tests.", timestamp: "10:18" },
  { id: "m4", author: "system", authorName: "Verification", body: "build: FAILED — type error in middleware.ts:88.", timestamp: "10:21" },
  { id: "m5", author: "user", authorName: "You", body: "Fix the middleware type — it accepts `string | undefined` now.", timestamp: "10:22" },
  { id: "m6", author: "agent", authorName: "Claude Code", body: "Fixed. Re-running build.", timestamp: "10:23" },
];

export const mockGraphNodes: ContextGraphNode[] = [
  { id: "n-task", label: "Task #042", kind: "task", x: 180, y: 80 },
  { id: "n-branch", label: "magic-link", kind: "branch", x: 60, y: 160 },
  { id: "n-claude", label: "Claude Code", kind: "engine", x: 300, y: 160 },
  { id: "n-auth", label: "src/auth/", kind: "file", x: 80, y: 240 },
  { id: "n-mid", label: "middleware.ts", kind: "file", x: 200, y: 250 },
  { id: "n-tests", label: "auth.test.ts", kind: "file", x: 320, y: 250 },
];

export const mockGraphEdges: ContextGraphEdge[] = [
  { from: "n-task", to: "n-branch" },
  { from: "n-task", to: "n-claude" },
  { from: "n-branch", to: "n-auth" },
  { from: "n-claude", to: "n-mid" },
  { from: "n-claude", to: "n-tests" },
];

export const mockSnapshot: SnapshotMetric[] = [
  { label: "Tasks in flight", value: "3", trend: "up" },
  { label: "Verification pass", value: "82%", trend: "flat" },
  { label: "Highest spend", value: "Claude Code · $4.18", trend: "up" },
];

export const mockActiveAgents: ActiveAgent[] = [
  { engineId: "claude-code", taskId: "task-042", status: "running" },
  { engineId: "codex-cli", taskId: "task-041", status: "blocked" },
];
```

---

## Task 28: `lib/utils.ts` — extend with helpers

**Files:**
- Modify: `lib/utils.ts`

- [ ] **Step 1: Read the current file**

Run: `cat lib/utils.ts`
Expected: the `cn()` helper from shadcn (see Task 10).

- [ ] **Step 2: Append a small async sleep helper used by the dev-mode dispatcher**

```ts
import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
```

Overwrite `lib/utils.ts` with the content above.

---

## Task 29: `lib/tauri.ts` — bridge wrapper + dev-mode mock dispatcher

**Files:**
- Create: `lib/tauri.ts`

- [ ] **Step 1: Write `lib/tauri.ts`**

```ts
import { isTauri } from "@tauri-apps/api/core";
import * as bindings from "./bindings";
import {
  mockEngines,
  mockProjects,
  mockTasks,
  mockVerification,
} from "./mock-data";
import { sleep } from "./utils";

type Bindings = typeof bindings;

const mockImpl: Partial<Bindings> = {
  listProjects: async () => {
    await sleep(50);
    return mockProjects;
  },
  listTasks: async (args: { projectId: string }) => {
    await sleep(50);
    return mockTasks.filter((t) => t.projectId === args.projectId);
  },
  getTask: async (args: { taskId: string }) => {
    await sleep(50);
    const task = mockTasks.find((t) => t.id === args.taskId);
    if (!task) {
      throw {
        code: "notFound",
        message: `task ${args.taskId} not found`,
        details: null,
      };
    }
    return task;
  },
  listEngines: async () => {
    await sleep(50);
    return mockEngines;
  },
  detectEngines: async () => {
    await sleep(80);
    return mockEngines;
  },
  listVerification: async (args: { taskId: string }) => {
    await sleep(50);
    return mockVerification.filter((v) => v.taskId === args.taskId);
  },
};

function pickImpl(): Bindings {
  if (typeof window !== "undefined" && isTauri()) {
    return bindings;
  }
  // Dev-mode: fall back to mocks. Cast through unknown because we are
  // intentionally providing only the subset we mock.
  return mockImpl as unknown as Bindings;
}

const impl = pickImpl();

export const tauri: Bindings = impl;
export type { Project, Task, EngineStatus, VerificationRun, AppError } from "./bindings";
```

> Note: `lib/bindings.ts` is generated by `tauri-specta` in Task 25 and is `.gitignore`d. If a contributor pulls the repo without it, `pnpm gen:bindings` recreates it.

---

## Task 30: Dev-mode dispatcher integration test

**Files:**
- Create: `lib/tauri.test.ts`
- Modify: `package.json` (add vitest)

- [ ] **Step 1: Install Vitest**

```bash
pnpm add -D vitest @vitejs/plugin-react jsdom
```

- [ ] **Step 2: Add a `vitest.config.ts`**

```ts
import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "jsdom",
    globals: true,
    include: ["**/*.test.ts", "**/*.test.tsx"],
    exclude: ["node_modules", ".next", "out", "src-tauri"],
  },
  resolve: {
    alias: {
      "@": new URL("./", import.meta.url).pathname,
    },
  },
});
```

- [ ] **Step 3: Add a `test` script to `package.json`**

Edit the `"scripts"` block to include:

```json
    "test": "vitest run",
    "test:watch": "vitest"
```

- [ ] **Step 4: Write the failing test**

`lib/tauri.test.ts`:

```ts
import { describe, it, expect } from "vitest";
import { tauri } from "./tauri";

describe("tauri bridge (dev-mode mock dispatcher)", () => {
  it("listProjects returns the default project", async () => {
    const result = await tauri.listProjects();
    expect(result).toHaveLength(1);
    expect(result[0].id).toBe("proj-default");
  });

  it("listTasks filters by projectId", async () => {
    const tasks = await tauri.listTasks({ projectId: "proj-default" });
    expect(tasks.length).toBeGreaterThan(0);
    expect(tasks.every((t) => t.projectId === "proj-default")).toBe(true);
  });

  it("getTask throws AppError-shaped object on miss", async () => {
    await expect(tauri.getTask({ taskId: "nope" })).rejects.toMatchObject({
      code: "notFound",
    });
  });

  it("listEngines returns both Claude Code and Codex CLI", async () => {
    const engines = await tauri.listEngines();
    expect(engines.map((e) => e.id).sort()).toEqual(["claude-code", "codex-cli"]);
  });
});
```

- [ ] **Step 5: Run the test — it should pass once bindings exist**

```bash
pnpm gen:bindings   # produces lib/bindings.ts if missing
pnpm test
```

Expected: 4 tests pass. If `lib/bindings.ts` isn't there or the import shape doesn't match the mock, fix `lib/mock-data.ts` to match the generated TS types — the boundary is canonical.

- [ ] **Step 6: Commit**

```bash
git add lib/tauri.ts lib/tauri.test.ts lib/mock-data.ts lib/types.ts lib/utils.ts vitest.config.ts package.json pnpm-lock.yaml
git commit -m "feat(bridge): typed tauri wrapper, mock dispatcher, vitest"
```

---

## Task 31: `stores/ui-store.ts` (Zustand UI cursor)

**Files:**
- Create: `stores/ui-store.ts`

- [ ] **Step 1: Write `stores/ui-store.ts`**

```bash
mkdir -p stores
```

```ts
import { create } from "zustand";

interface UiState {
  activeProjectId: string;
  activeTaskId: string | null;
  agentManagerOpen: boolean;
  setActiveTask: (taskId: string | null) => void;
  setActiveProject: (projectId: string) => void;
  toggleAgentManager: () => void;
}

export const useUiStore = create<UiState>((set) => ({
  activeProjectId: "proj-default",
  activeTaskId: "task-042",
  agentManagerOpen: false,
  setActiveTask: (taskId) => set({ activeTaskId: taskId }),
  setActiveProject: (projectId) => set({ activeProjectId: projectId, activeTaskId: null }),
  toggleAgentManager: () => set((s) => ({ agentManagerOpen: !s.agentManagerOpen })),
}));
```

---

## Task 32: `stores/task-store.ts` (streaming-log buffer)

**Files:**
- Create: `stores/task-store.ts`

- [ ] **Step 1: Write `stores/task-store.ts`**

```ts
import { create } from "zustand";

interface LogLine {
  id: string;
  taskId: string;
  body: string;
  timestamp: string;
  kind: "stdout" | "stderr" | "system";
}

interface TaskState {
  streamingLog: Record<string, LogLine[]>;
  appendLog: (taskId: string, line: Omit<LogLine, "taskId">) => void;
  clearLog: (taskId: string) => void;
}

export const useTaskStore = create<TaskState>((set) => ({
  streamingLog: {
    "task-042": [
      { id: "l1", taskId: "task-042", body: "Reading src/auth/jwt.ts", timestamp: "10:14:02", kind: "stdout" },
      { id: "l2", taskId: "task-042", body: "Creating src/auth/magic-link.ts", timestamp: "10:14:11", kind: "stdout" },
      { id: "l3", taskId: "task-042", body: "Running pnpm test...", timestamp: "10:18:00", kind: "system" },
      { id: "l4", taskId: "task-042", body: "142 passed, 0 failed", timestamp: "10:18:18", kind: "stdout" },
      { id: "l5", taskId: "task-042", body: "build: type error in middleware.ts:88", timestamp: "10:21:04", kind: "stderr" },
    ],
  },
  appendLog: (taskId, line) =>
    set((s) => ({
      streamingLog: {
        ...s.streamingLog,
        [taskId]: [...(s.streamingLog[taskId] ?? []), { ...line, taskId }],
      },
    })),
  clearLog: (taskId) =>
    set((s) => ({ streamingLog: { ...s.streamingLog, [taskId]: [] } })),
}));
```

---

## Task 33: `stores/engine-store.ts`

**Files:**
- Create: `stores/engine-store.ts`

- [ ] **Step 1: Write `stores/engine-store.ts`**

```ts
import { create } from "zustand";

interface EngineUiState {
  preferredEngineId: string;
  setPreferredEngine: (id: string) => void;
}

export const useEngineStore = create<EngineUiState>((set) => ({
  preferredEngineId: "claude-code",
  setPreferredEngine: (id) => set({ preferredEngineId: id }),
}));
```

---

## Task 34: Extend `app/providers.tsx` with TanStack Query

**Files:**
- Modify: `app/providers.tsx`

- [ ] **Step 1: Rewrite `app/providers.tsx`**

```tsx
"use client";

import { ThemeProvider } from "next-themes";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";

export function Providers({ children }: { children: ReactNode }) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 30_000,
            refetchOnWindowFocus: false,
            retry: 1,
          },
        },
      }),
  );

  return (
    <ThemeProvider attribute="class" defaultTheme="dark" enableSystem disableTransitionOnChange>
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    </ThemeProvider>
  );
}
```

---

## Task 35: Feature hooks (TanStack Query wrappers)

**Files:**
- Create: `features/tasks/use-tasks.ts`
- Create: `features/projects/use-projects.ts`
- Create: `features/engines/use-engines.ts`
- Create: `features/verification/use-verification.ts`

- [ ] **Step 1: Create directories**

```bash
mkdir -p features/tasks features/projects features/engines features/verification
```

- [ ] **Step 2: Write `features/projects/use-projects.ts`**

```ts
import { useQuery } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function useProjects() {
  return useQuery({
    queryKey: ["projects"],
    queryFn: () => tauri.listProjects(),
  });
}
```

- [ ] **Step 3: Write `features/tasks/use-tasks.ts`**

```ts
import { useQuery } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function useTasks(projectId: string) {
  return useQuery({
    queryKey: ["tasks", projectId],
    queryFn: () => tauri.listTasks({ projectId }),
    enabled: !!projectId,
  });
}

export function useTask(taskId: string | null) {
  return useQuery({
    queryKey: ["task", taskId],
    queryFn: () => tauri.getTask({ taskId: taskId! }),
    enabled: !!taskId,
  });
}
```

- [ ] **Step 4: Write `features/engines/use-engines.ts`**

```ts
import { useQuery } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function useEngines() {
  return useQuery({
    queryKey: ["engines"],
    queryFn: () => tauri.listEngines(),
  });
}
```

- [ ] **Step 5: Write `features/verification/use-verification.ts`**

```ts
import { useQuery } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function useVerification(taskId: string | null) {
  return useQuery({
    queryKey: ["verification", taskId],
    queryFn: () => tauri.listVerification({ taskId: taskId! }),
    enabled: !!taskId,
  });
}
```

- [ ] **Step 6: Commit**

```bash
git add stores features app/providers.tsx
git commit -m "feat(state): Zustand stores, TanStack Query provider, feature hooks"
```

---

## Task 36: Quick smoke — typecheck after wiring

- [ ] **Step 1: Run typecheck**

Run: `pnpm typecheck`
Expected: PASS. If anything fails, the boundary types in `lib/bindings.ts` likely don't line up with `lib/mock-data.ts` — fix `mock-data.ts` field names (camelCase) to match the generated TS exactly.

---

## Task 37: `PanelFrame` component

**Files:**
- Create: `components/layout/panel-frame.tsx`

- [ ] **Step 1: Write `components/layout/panel-frame.tsx`**

```tsx
import type { ReactNode } from "react";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";

interface PanelFrameProps {
  title: string;
  subtitle?: string;
  badge?: string;
  actions?: ReactNode;
  className?: string;
  bodyClassName?: string;
  children: ReactNode;
}

export function PanelFrame({
  title,
  subtitle,
  badge,
  actions,
  className,
  bodyClassName,
  children,
}: PanelFrameProps) {
  return (
    <section className={cn("panel-surface flex flex-col overflow-hidden", className)}>
      <header className="flex items-center justify-between gap-3 border-b border-border/60 px-4 py-3">
        <div className="flex items-center gap-2">
          <h2 className="text-sm font-medium text-foreground">{title}</h2>
          {badge && (
            <Badge variant="secondary" className="text-[10px] uppercase tracking-wider">
              {badge}
            </Badge>
          )}
        </div>
        {actions && <div className="flex items-center gap-1">{actions}</div>}
      </header>
      {subtitle && (
        <p className="border-b border-border/60 px-4 py-2 text-xs text-muted-foreground">{subtitle}</p>
      )}
      <div className={cn("flex-1 overflow-auto p-4", bodyClassName)}>{children}</div>
    </section>
  );
}
```

---

## Task 38: `AppHeader` component

**Files:**
- Create: `components/layout/app-header.tsx`

- [ ] **Step 1: Write `components/layout/app-header.tsx`**

```tsx
"use client";

import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { ThemeToggle } from "./theme-toggle";

export function AppHeader() {
  return (
    <header className="flex h-14 items-center justify-between gap-4 border-b border-border/60 bg-background/80 px-5 backdrop-blur">
      <div className="flex items-center gap-3">
        <div className="flex h-8 w-8 items-center justify-center rounded-md bg-primary/20 font-mono text-sm font-semibold text-primary">
          AS
        </div>
        <div className="flex flex-col leading-tight">
          <span className="text-sm font-semibold">AI Software Studio</span>
          <span className="text-[11px] text-muted-foreground">Local-first</span>
        </div>
      </div>

      <div className="hidden flex-1 flex-col items-center text-center md:flex">
        <h1 className="text-base font-semibold tracking-tight">The AI-first IDE</h1>
        <p className="text-[11px] text-muted-foreground">
          Watch first. Evidence-first. Human accountable.
        </p>
      </div>

      <div className="flex items-center gap-2">
        <div className="flex items-center gap-1 rounded-md border border-border/60 bg-muted/40 p-0.5 text-xs">
          <button className="rounded-sm bg-card px-2 py-1 font-medium shadow-sm">Studio</button>
          <button className="px-2 py-1 text-muted-foreground hover:text-foreground">Trace</button>
        </div>
        <Separator orientation="vertical" className="h-6" />
        <Badge variant="outline" className="hidden text-[10px] sm:inline-flex">v0.0.1</Badge>
        <ThemeToggle />
        <Avatar className="h-7 w-7">
          <AvatarFallback className="text-[11px]">R</AvatarFallback>
        </Avatar>
      </div>
    </header>
  );
}
```

---

## Task 39: `DashboardShell` (3-column grid)

**Files:**
- Create: `components/layout/dashboard-shell.tsx`

- [ ] **Step 1: Write `components/layout/dashboard-shell.tsx`**

```tsx
"use client";

import type { ReactNode } from "react";
import { AppHeader } from "./app-header";

interface DashboardShellProps {
  left: ReactNode;       // Task Board + Engineering Snapshot
  center: ReactNode;     // Agent Workspace + Review Room
  right: ReactNode;      // Context Graph + Conversation + Agent Manager
}

export function DashboardShell({ left, center, right }: DashboardShellProps) {
  return (
    <div className="grid h-screen grid-rows-[auto_1fr] bg-background">
      <AppHeader />
      <div className="grid min-h-0 grid-cols-[280px_1fr_360px] gap-3 p-3">
        <div className="grid min-h-0 grid-rows-[1fr_auto] gap-3">{left}</div>
        <div className="grid min-h-0 grid-rows-[1fr_auto] gap-3">{center}</div>
        <div className="grid min-h-0 grid-rows-[auto_1fr_auto] gap-3">{right}</div>
      </div>
    </div>
  );
}
```

---

## Task 40: Wire `app/page.tsx` to dashboard (panels still missing — placeholders for now)

**Files:**
- Modify: `app/page.tsx`

- [ ] **Step 1: Rewrite `app/page.tsx`** (panels are placeholders here; they get real bodies in Tasks 42–48)

```tsx
"use client";

import { DashboardShell } from "@/components/layout/dashboard-shell";
import { PanelFrame } from "@/components/layout/panel-frame";

function Placeholder({ name }: { name: string }) {
  return (
    <PanelFrame title={name} badge="WIP">
      <p className="text-sm text-muted-foreground">Panel content arrives in the next task.</p>
    </PanelFrame>
  );
}

export default function HomePage() {
  return (
    <DashboardShell
      left={
        <>
          <Placeholder name="Task Board" />
          <Placeholder name="Engineering Snapshot" />
        </>
      }
      center={
        <>
          <Placeholder name="Agent Workspace" />
          <Placeholder name="Review Room" />
        </>
      }
      right={
        <>
          <Placeholder name="Context Graph" />
          <Placeholder name="Conversation" />
          <Placeholder name="Agent Manager" />
        </>
      }
    />
  );
}
```

- [ ] **Step 2: Run dev and verify the 3-column shell**

Run: `pnpm dev`
Open `http://localhost:3000`.
Expected: Header at top, three columns of WIP placeholder panels, theme toggle works. Stop the server.

- [ ] **Step 3: Commit**

```bash
git add components/layout app/page.tsx
git commit -m "feat(layout): PanelFrame, AppHeader, DashboardShell, 3-column page"
```

---

## Task 41: Quick visual sanity in Tauri before populating panels

**Files:** none

- [ ] **Step 1: Run `pnpm tauri dev`**

```bash
pnpm tauri dev
```

Expected: Rust builds (may take a minute the first time), a desktop window opens at 1440×900 showing the same dashboard shell with WIP placeholders. Close the window when satisfied (Cmd+Q on macOS).

If the window doesn't open: check `src-tauri/tauri.conf.json`'s `frontendDist` (`../out`) and `devUrl`. If `frontendDist` doesn't exist yet, that's fine in `tauri dev` because it uses `devUrl`; the path matters for `tauri build`.

---

## Task 42: Task Board panel

**Files:**
- Create: `components/panels/task-board/index.tsx`
- Create: `components/panels/task-board/task-card.tsx`

- [ ] **Step 1: Create folder**

```bash
mkdir -p components/panels/task-board
```

- [ ] **Step 2: Write `components/panels/task-board/task-card.tsx`**

```tsx
"use client";

import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import type { Task } from "@/lib/tauri";

const statusColor: Record<string, string> = {
  draft: "bg-muted text-muted-foreground",
  running: "bg-warning/20 text-warning",
  reviewReady: "bg-primary/20 text-primary",
  approved: "bg-success/20 text-success",
  changesRequested: "bg-destructive/20 text-destructive",
};

interface TaskCardProps {
  task: Task;
  active: boolean;
  onSelect: () => void;
}

export function TaskCard({ task, active, onSelect }: TaskCardProps) {
  return (
    <button
      onClick={onSelect}
      className={cn(
        "w-full rounded-lg border border-border/60 bg-card/60 p-3 text-left transition-colors",
        "hover:bg-card focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
        active && "border-primary/60 bg-primary/10",
      )}
    >
      <div className="flex items-start justify-between gap-2">
        <span className="font-mono text-[10px] uppercase tracking-wider text-muted-foreground">
          #{task.id.replace("task-", "")}
        </span>
        <Badge className={cn("text-[10px]", statusColor[task.status] ?? "bg-muted")}>{task.status}</Badge>
      </div>
      <p className="mt-1.5 line-clamp-2 text-sm leading-snug">{task.title}</p>
      {task.selectedEngine && (
        <p className="mt-2 text-[11px] text-muted-foreground">via {task.selectedEngine}</p>
      )}
    </button>
  );
}
```

- [ ] **Step 3: Write `components/panels/task-board/index.tsx`**

```tsx
"use client";

import { Plus } from "lucide-react";
import { PanelFrame } from "@/components/layout/panel-frame";
import { Button } from "@/components/ui/button";
import { useTasks } from "@/features/tasks/use-tasks";
import { useUiStore } from "@/stores/ui-store";
import { TaskCard } from "./task-card";

export function TaskBoardPanel() {
  const activeProjectId = useUiStore((s) => s.activeProjectId);
  const activeTaskId = useUiStore((s) => s.activeTaskId);
  const setActiveTask = useUiStore((s) => s.setActiveTask);
  const { data: tasks = [], isLoading } = useTasks(activeProjectId);

  return (
    <PanelFrame
      title="Task Board"
      badge="Initiative"
      actions={
        <Button size="icon" variant="ghost" className="h-7 w-7" aria-label="Add task">
          <Plus className="h-4 w-4" />
        </Button>
      }
      bodyClassName="space-y-2"
    >
      {isLoading && <p className="text-xs text-muted-foreground">Loading tasks…</p>}
      {tasks.map((task) => (
        <TaskCard
          key={task.id}
          task={task}
          active={task.id === activeTaskId}
          onSelect={() => setActiveTask(task.id)}
        />
      ))}
    </PanelFrame>
  );
}
```

---

## Task 43: Engineering Snapshot panel

**Files:**
- Create: `components/panels/engineering-snapshot/index.tsx`

- [ ] **Step 1: Create folder + file**

```bash
mkdir -p components/panels/engineering-snapshot
```

```tsx
"use client";

import { ArrowDown, ArrowUp, Minus } from "lucide-react";
import { PanelFrame } from "@/components/layout/panel-frame";
import { mockSnapshot } from "@/lib/mock-data";

const trendIcon = {
  up: <ArrowUp className="h-3 w-3 text-success" />,
  down: <ArrowDown className="h-3 w-3 text-destructive" />,
  flat: <Minus className="h-3 w-3 text-muted-foreground" />,
};

export function EngineeringSnapshotPanel() {
  return (
    <PanelFrame title="Engineering Snapshot" bodyClassName="space-y-2">
      {mockSnapshot.map((m) => (
        <div
          key={m.label}
          className="flex items-center justify-between rounded-md border border-border/40 bg-muted/40 px-3 py-2"
        >
          <div className="flex flex-col">
            <span className="text-[11px] text-muted-foreground">{m.label}</span>
            <span className="text-sm font-medium">{m.value}</span>
          </div>
          {m.trend && trendIcon[m.trend]}
        </div>
      ))}
    </PanelFrame>
  );
}
```

---

## Task 44: Agent Workspace panel

**Files:**
- Create: `components/panels/agent-workspace/index.tsx`
- Create: `components/panels/agent-workspace/acceptance-list.tsx`
- Create: `components/panels/agent-workspace/activity-log.tsx`

- [ ] **Step 1: Create folder**

```bash
mkdir -p components/panels/agent-workspace
```

- [ ] **Step 2: Write `acceptance-list.tsx`**

```tsx
"use client";

import { Check, Circle } from "lucide-react";
import { cn } from "@/lib/utils";
import type { Task } from "@/lib/tauri";

export function AcceptanceList({ items }: { items: Task["acceptanceCriteria"] }) {
  if (items.length === 0) {
    return <p className="text-xs text-muted-foreground">No acceptance criteria.</p>;
  }
  return (
    <ul className="space-y-1.5">
      {items.map((item) => (
        <li key={item.id} className="flex items-start gap-2 text-sm">
          {item.satisfied ? (
            <Check className="mt-0.5 h-4 w-4 shrink-0 text-success" />
          ) : (
            <Circle className="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" />
          )}
          <span className={cn(item.satisfied && "text-muted-foreground line-through")}>{item.label}</span>
        </li>
      ))}
    </ul>
  );
}
```

- [ ] **Step 3: Write `activity-log.tsx`**

```tsx
"use client";

import { cn } from "@/lib/utils";
import { useTaskStore } from "@/stores/task-store";

const kindColor = {
  stdout: "text-foreground",
  stderr: "text-destructive",
  system: "text-primary",
};

export function ActivityLog({ taskId }: { taskId: string }) {
  const lines = useTaskStore((s) => s.streamingLog[taskId] ?? []);
  if (lines.length === 0) {
    return <p className="text-xs text-muted-foreground">No activity yet.</p>;
  }
  return (
    <div className="rounded-md border border-border/60 bg-muted/30 p-3 font-mono text-[11px] leading-relaxed">
      {lines.map((l) => (
        <div key={l.id} className="flex gap-2">
          <span className="text-muted-foreground">{l.timestamp}</span>
          <span className={cn(kindColor[l.kind])}>{l.body}</span>
        </div>
      ))}
    </div>
  );
}
```

- [ ] **Step 4: Write `index.tsx`**

```tsx
"use client";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { PanelFrame } from "@/components/layout/panel-frame";
import { useUiStore } from "@/stores/ui-store";
import { useTask } from "@/features/tasks/use-tasks";
import { AcceptanceList } from "./acceptance-list";
import { ActivityLog } from "./activity-log";

export function AgentWorkspacePanel() {
  const activeTaskId = useUiStore((s) => s.activeTaskId);
  const { data: task } = useTask(activeTaskId);

  if (!activeTaskId || !task) {
    return (
      <PanelFrame title="Agent Workspace" subtitle="Select a task from the Task Board to begin.">
        <p className="text-xs text-muted-foreground">No task selected.</p>
      </PanelFrame>
    );
  }

  return (
    <PanelFrame
      title="Agent Workspace"
      subtitle="Watch first. Evidence-first. Human accountable."
      badge={task.status}
      actions={
        <div className="flex items-center gap-1">
          <Button size="sm" variant="ghost">Request Changes</Button>
          <Button size="sm">Approve</Button>
        </div>
      }
    >
      <div className="space-y-4">
        <div>
          <div className="flex items-center gap-2">
            <span className="font-mono text-xs text-muted-foreground">Task #{task.id.replace("task-", "")}</span>
            <Badge variant="outline" className="text-[10px]">{task.risk}</Badge>
          </div>
          <h3 className="mt-1 text-base font-semibold">{task.title}</h3>
          <p className="mt-1 text-sm text-muted-foreground">{task.description}</p>
        </div>

        <div>
          <h4 className="mb-2 text-xs font-medium uppercase tracking-wider text-muted-foreground">
            Acceptance Criteria
          </h4>
          <AcceptanceList items={task.acceptanceCriteria} />
        </div>

        <div>
          <h4 className="mb-2 text-xs font-medium uppercase tracking-wider text-muted-foreground">
            Activity
          </h4>
          <ActivityLog taskId={task.id} />
        </div>
      </div>
    </PanelFrame>
  );
}
```

---

## Task 45: Review Room panel

**Files:**
- Create: `components/panels/review-room/index.tsx`
- Create: `components/panels/review-room/status-pill.tsx`
- Create: `components/panels/review-room/evidence-artifacts.tsx`

- [ ] **Step 1: Create folder**

```bash
mkdir -p components/panels/review-room
```

- [ ] **Step 2: Write `status-pill.tsx`**

```tsx
"use client";

import { Check, CircleAlert, Clock, Minus, X } from "lucide-react";
import { cn } from "@/lib/utils";
import type { VerificationRun } from "@/lib/tauri";

type Status = VerificationRun["checks"][number]["status"];

const meta: Record<Status, { icon: typeof Check; className: string }> = {
  passed:    { icon: Check,       className: "bg-success/15 text-success" },
  failed:    { icon: X,           className: "bg-destructive/15 text-destructive" },
  warning:   { icon: CircleAlert, className: "bg-warning/15 text-warning" },
  running:   { icon: Clock,       className: "bg-primary/15 text-primary" },
  notRun:    { icon: Minus,       className: "bg-muted text-muted-foreground" },
  skipped:   { icon: Minus,       className: "bg-muted text-muted-foreground" },
};

export function StatusPill({ kind, status }: { kind: string; status: Status }) {
  const { icon: Icon, className } = meta[status];
  return (
    <div className={cn("flex items-center justify-between gap-2 rounded-md border border-border/40 px-3 py-2", className)}>
      <span className="text-xs font-medium uppercase tracking-wider">{kind}</span>
      <Icon className="h-4 w-4" />
    </div>
  );
}
```

- [ ] **Step 3: Write `evidence-artifacts.tsx`**

```tsx
"use client";

const bars = [3, 7, 5, 9, 4, 8, 6, 10, 5, 7, 9, 6, 8];

export function EvidenceArtifacts() {
  return (
    <div className="rounded-md border border-border/40 bg-muted/30 p-3">
      <h4 className="mb-2 text-[10px] font-medium uppercase tracking-wider text-muted-foreground">
        Evidence Artifacts
      </h4>
      <div className="flex h-12 items-end gap-1">
        {bars.map((h, i) => (
          <div
            key={i}
            style={{ height: `${h * 8}%` }}
            className="w-2 rounded-sm bg-primary/60"
          />
        ))}
      </div>
    </div>
  );
}
```

- [ ] **Step 4: Write `index.tsx`**

```tsx
"use client";

import { PanelFrame } from "@/components/layout/panel-frame";
import { useUiStore } from "@/stores/ui-store";
import { useVerification } from "@/features/verification/use-verification";
import { StatusPill } from "./status-pill";
import { EvidenceArtifacts } from "./evidence-artifacts";

export function ReviewRoomPanel() {
  const activeTaskId = useUiStore((s) => s.activeTaskId);
  const { data: runs = [] } = useVerification(activeTaskId);
  const latest = runs[0];

  return (
    <PanelFrame title="Review Room" badge="Verification">
      <div className="space-y-3">
        {!latest && (
          <p className="text-xs text-muted-foreground">No verification run yet.</p>
        )}
        {latest && (
          <div className="grid grid-cols-2 gap-2 md:grid-cols-5">
            {latest.checks.map((c) => (
              <StatusPill key={c.kind} kind={c.kind} status={c.status} />
            ))}
          </div>
        )}
        <EvidenceArtifacts />
      </div>
    </PanelFrame>
  );
}
```

---

## Task 46: Context Graph panel (with Active Agents)

**Files:**
- Create: `components/panels/context-graph/index.tsx`
- Create: `components/panels/context-graph/active-agents.tsx`
- Create: `components/panels/context-graph/graph-svg.tsx`

- [ ] **Step 1: Create folder**

```bash
mkdir -p components/panels/context-graph
```

- [ ] **Step 2: Write `active-agents.tsx`**

```tsx
"use client";

import { Badge } from "@/components/ui/badge";
import { useEngines } from "@/features/engines/use-engines";
import { mockActiveAgents } from "@/lib/mock-data";

export function ActiveAgents() {
  const { data: engines = [] } = useEngines();
  return (
    <div className="space-y-1.5">
      <h4 className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">
        Active Agents
      </h4>
      {engines.map((e) => {
        const agent = mockActiveAgents.find((a) => a.engineId === e.id);
        return (
          <div
            key={e.id}
            className="flex items-center justify-between rounded-md border border-border/40 bg-muted/30 px-3 py-2 text-xs"
          >
            <div className="flex flex-col">
              <span className="font-medium">{e.name}</span>
              <span className="text-[10px] text-muted-foreground">
                {e.version ? `v${e.version}` : "no version"}
              </span>
            </div>
            <Badge
              variant={e.status === "ready" ? "default" : "outline"}
              className="text-[10px]"
            >
              {agent ? agent.status : e.status}
            </Badge>
          </div>
        );
      })}
    </div>
  );
}
```

- [ ] **Step 3: Write `graph-svg.tsx`**

```tsx
"use client";

import { mockGraphEdges, mockGraphNodes } from "@/lib/mock-data";

const colorFor: Record<string, string> = {
  task: "var(--primary)",
  engine: "var(--accent)",
  branch: "var(--secondary)",
  file: "var(--muted-foreground)",
};

export function GraphSvg() {
  return (
    <svg viewBox="0 0 380 300" className="h-44 w-full">
      {mockGraphEdges.map((e, i) => {
        const from = mockGraphNodes.find((n) => n.id === e.from);
        const to = mockGraphNodes.find((n) => n.id === e.to);
        if (!from || !to) return null;
        return (
          <line
            key={i}
            x1={from.x}
            y1={from.y}
            x2={to.x}
            y2={to.y}
            stroke="var(--border)"
            strokeWidth={1}
          />
        );
      })}
      {mockGraphNodes.map((n) => (
        <g key={n.id} transform={`translate(${n.x}, ${n.y})`}>
          <circle r={10} fill={colorFor[n.kind] ?? "var(--muted)"} opacity={0.85} />
          <text
            y={22}
            textAnchor="middle"
            className="fill-foreground text-[9px]"
          >
            {n.label}
          </text>
        </g>
      ))}
    </svg>
  );
}
```

- [ ] **Step 4: Write `index.tsx`**

```tsx
"use client";

import { PanelFrame } from "@/components/layout/panel-frame";
import { ActiveAgents } from "./active-agents";
import { GraphSvg } from "./graph-svg";

export function ContextGraphPanel() {
  return (
    <PanelFrame title="Context Graph" badge="Live">
      <div className="space-y-4">
        <ActiveAgents />
        <GraphSvg />
      </div>
    </PanelFrame>
  );
}
```

---

## Task 47: Conversation panel

**Files:**
- Create: `components/panels/conversation/index.tsx`

- [ ] **Step 1: Create folder + file**

```bash
mkdir -p components/panels/conversation
```

```tsx
"use client";

import { cn } from "@/lib/utils";
import { PanelFrame } from "@/components/layout/panel-frame";
import { mockConversation } from "@/lib/mock-data";

const authorColor = {
  user: "border-primary/40 bg-primary/10",
  agent: "border-border/60 bg-muted/40",
  system: "border-warning/40 bg-warning/10",
};

export function ConversationPanel() {
  return (
    <PanelFrame title="Comment / Conversation" badge="Timeline" bodyClassName="space-y-2">
      {mockConversation.map((m) => (
        <div key={m.id} className={cn("rounded-md border p-2.5 text-xs", authorColor[m.author])}>
          <div className="mb-1 flex items-center justify-between text-[10px] text-muted-foreground">
            <span className="font-medium">{m.authorName}</span>
            <span>{m.timestamp}</span>
          </div>
          <p className="leading-relaxed">{m.body}</p>
        </div>
      ))}
    </PanelFrame>
  );
}
```

---

## Task 48: Agent Manager panel

**Files:**
- Create: `components/panels/agent-manager/index.tsx`

- [ ] **Step 1: Create folder + file**

```bash
mkdir -p components/panels/agent-manager
```

```tsx
"use client";

import { Power } from "lucide-react";
import { PanelFrame } from "@/components/layout/panel-frame";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useEngines } from "@/features/engines/use-engines";

const statusBadge = {
  ready: "bg-success/20 text-success",
  notInstalled: "bg-muted text-muted-foreground",
  notAuthenticated: "bg-warning/20 text-warning",
  detected: "bg-primary/20 text-primary",
  error: "bg-destructive/20 text-destructive",
};

export function AgentManagerPanel() {
  const { data: engines = [] } = useEngines();
  return (
    <PanelFrame
      title="Agent Manager"
      badge="Engines"
      actions={
        <Button size="sm" variant="ghost" className="h-7">
          <Power className="mr-1 h-3 w-3" /> Re-detect
        </Button>
      }
      bodyClassName="space-y-2"
    >
      {engines.map((e) => (
        <div
          key={e.id}
          className="flex items-center justify-between rounded-md border border-border/40 bg-muted/30 px-3 py-2"
        >
          <div className="flex flex-col">
            <span className="text-sm font-medium">{e.name}</span>
            <span className="text-[10px] text-muted-foreground">
              {e.binaryPath ?? "binary not found"}
            </span>
          </div>
          <Badge className={`text-[10px] ${statusBadge[e.status] ?? "bg-muted"}`}>
            {e.status}
          </Badge>
        </div>
      ))}
    </PanelFrame>
  );
}
```

---

## Task 49: Wire all panels into `app/page.tsx`

**Files:**
- Modify: `app/page.tsx`

- [ ] **Step 1: Rewrite `app/page.tsx`**

```tsx
"use client";

import { DashboardShell } from "@/components/layout/dashboard-shell";
import { TaskBoardPanel } from "@/components/panels/task-board";
import { EngineeringSnapshotPanel } from "@/components/panels/engineering-snapshot";
import { AgentWorkspacePanel } from "@/components/panels/agent-workspace";
import { ReviewRoomPanel } from "@/components/panels/review-room";
import { ContextGraphPanel } from "@/components/panels/context-graph";
import { ConversationPanel } from "@/components/panels/conversation";
import { AgentManagerPanel } from "@/components/panels/agent-manager";

export default function HomePage() {
  return (
    <DashboardShell
      left={
        <>
          <TaskBoardPanel />
          <EngineeringSnapshotPanel />
        </>
      }
      center={
        <>
          <AgentWorkspacePanel />
          <ReviewRoomPanel />
        </>
      }
      right={
        <>
          <ContextGraphPanel />
          <ConversationPanel />
          <AgentManagerPanel />
        </>
      }
    />
  );
}
```

- [ ] **Step 2: Run dev and visually verify**

Run: `pnpm dev`
Open `http://localhost:3000`.

Expected: All 7 panels render with mock data. Clicking a task in the Task Board updates the Agent Workspace and Review Room. Theme toggle still works. Stop the server.

- [ ] **Step 3: Commit**

```bash
git add components/panels app/page.tsx
git commit -m "feat(panels): all 7 dashboard panels wired with mock data"
```

---

## Task 50: Full verification — typecheck, lint, test, build

**Files:** none

- [ ] **Step 1: Run typecheck**

```bash
pnpm typecheck
```
Expected: PASS.

- [ ] **Step 2: Run lint**

```bash
pnpm lint
```
Expected: PASS or only the known shadcn warnings about React component naming. If it surfaces real warnings (unused imports, missing keys), fix them inline and re-run.

- [ ] **Step 3: Run unit tests**

```bash
pnpm test
```
Expected: 4 tests pass (from Task 30).

- [ ] **Step 4: Run static build**

```bash
pnpm build
```
Expected: `out/` is created. Inspect with `ls out/` — should see `index.html`, `_next/`, etc.

---

## Task 51: Tauri smoke build

**Files:** none

- [ ] **Step 1: `pnpm tauri dev` final check**

```bash
pnpm tauri dev
```
Expected: Desktop window opens with the full dashboard, all panels populated, toggle works, task selection works.

Close the window when satisfied.

- [ ] **Step 2: `pnpm tauri build` smoke**

```bash
pnpm tauri build
```
Expected: On macOS, produces an `.app` bundle and an unsigned `.dmg` under `src-tauri/target/release/bundle/`. On Linux, produces an `.AppImage` and/or `.deb`. This step can take 5–15 minutes. If the build fails for code-signing reasons (macOS), confirm `tauri.conf.json` has no `signing` block set — we explicitly skip signing in this scaffold.

- [ ] **Step 3: Confirm the bundle runs**

On macOS: `open src-tauri/target/release/bundle/macos/AI\ Software\ Studio.app`
Expected: App launches and shows the dashboard.

Close the app.

---

## Task 52: Update README with "Running locally" section

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Read existing README**

Run: `cat README.md`
Expected: The existing intro about AI Software Studio.

- [ ] **Step 2: Append a "Running locally" section**

Use the Edit tool to add this block at the end of `README.md`:

```markdown

## Running Locally

**Prerequisites:** Node.js ≥ 20, pnpm ≥ 9, Rust ≥ 1.78 (install via `rustup`), Xcode Command Line Tools (macOS) or `build-essential` (Linux).

```bash
pnpm install
pnpm gen:bindings   # generates lib/bindings.ts from Rust types
pnpm dev            # browser-only iteration via mock dispatcher
pnpm tauri:dev      # full desktop window with Rust backend
```

Other scripts:

- `pnpm test` — unit tests (vitest)
- `pnpm typecheck` — TypeScript strict check
- `pnpm lint` — ESLint
- `pnpm build` — static export to `out/`
- `pnpm tauri:build` — desktop bundle (`.app` on macOS, `.AppImage`/`.deb` on Linux)

`lib/bindings.ts` is generated from Rust and `.gitignore`d. If your IDE complains it doesn't exist, run `pnpm gen:bindings`.

```

- [ ] **Step 3: Final commit**

```bash
git add README.md
git commit -m "docs: add Running Locally section to README"
```

- [ ] **Step 4: Show final state**

```bash
git log --oneline
git status
```
Expected: A clean working tree with a tidy chain of commits.

---

## Self-Review

After all tasks are complete, the scaffold's Definition of Done from the spec is met:

| DoD item | Verified by |
|---|---|
| Repo bootstrap (guarded git init) | Task 1 |
| pnpm install clean | Task 3, Task 6 |
| pnpm dev renders dashboard + theme toggle | Task 49 step 2 |
| pnpm tauri dev opens desktop window | Task 51 step 1 |
| pnpm typecheck passes | Task 50 step 1 |
| pnpm lint passes | Task 50 step 2 |
| pnpm build produces out/ | Task 50 step 4 |
| pnpm tauri build smoke-builds bundle | Task 51 step 2 |
| Visual fidelity to ui.png | Tasks 42–48 + visual checks in 49/51 |

**Type consistency check:** `Task`, `Project`, `EngineStatus`, `VerificationRun`, `AppError` are defined once in `src-tauri/src/models.rs` and `error.rs`, surface in TS via the generated `lib/bindings.ts`, and are re-exported from `lib/tauri.ts`. Mock data in `lib/mock-data.ts` uses camelCase keys matching the generated TS. All panel components import types via `@/lib/tauri`. No drift.

**Placeholder scan:** No "TBD" / "TODO: implement later" in steps. Placeholder Rust modules (`core/`, `db/`, `git/`, etc.) are intentional architectural seams — not unfinished plan steps — and each carries a `// TODO: implement in Phase ...` comment per the spec.
