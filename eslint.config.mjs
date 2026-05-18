import { FlatCompat } from "@eslint/eslintrc";

const compat = new FlatCompat({ baseDirectory: import.meta.dirname });

const config = [
  ...compat.extends("next/core-web-vitals", "next/typescript"),
  {
    ignores: ["src-tauri/**", "out/**", ".next/**", "node_modules/**", ".worktrees/**", "lib/bindings.ts", "next-env.d.ts"],
  },
];

export default config;
