import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [sveltekit()],

  // Unit-test coverage, written by `pnpm test:coverage` (see `just coverage`).
  test: {
    coverage: {
      provider: "v8",
      include: ["src/**"],
      reporter: ["text", "lcov", "json-summary"],
      reportsDirectory: "coverage/frontend",
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // This replaces Vite's default ignore list, so preserve generated/dependency exclusions.
      ignored: [
        "**/.git/**",
        "**/.pnpm-store/**",
        "**/build/**",
        "**/node_modules/**",
        "**/src-tauri/**",
        "**/target/**",
      ],
    },
  },
}));
