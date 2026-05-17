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
