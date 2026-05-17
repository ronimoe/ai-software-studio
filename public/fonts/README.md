# Local fonts

Place these files here:

- `Inter.woff2` — Inter Variable (download from https://rsms.me/inter/ or fontsource)
- `JetBrainsMono.woff2` — JetBrains Mono Variable (download from fontsource)

Both are referenced from `app/layout.tsx` via `next/font/local`. Builds will fail if these files are missing — that's intentional. Local fonts keep `pnpm build` and `pnpm tauri build` working without network access.
