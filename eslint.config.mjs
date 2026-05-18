import nextCoreWebVitals from "eslint-config-next/core-web-vitals";
import nextTypescript from "eslint-config-next/typescript";

const config = [
  ...nextCoreWebVitals,
  ...nextTypescript,
  {
    ignores: ["src-tauri/**", "out/**", ".next/**", "node_modules/**", ".worktrees/**", "lib/bindings.ts", "next-env.d.ts"],
  },
];

export default config;
