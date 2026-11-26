import js from "@eslint/js";
import { defineConfig } from "eslint/config";
import svelte from "eslint-plugin-svelte";
import globals from "globals";
import ts from "typescript-eslint";

import svelteConfig from "./svelte.config.js";

export default defineConfig(
  {
    ignores: [
      ".svelte-kit/**",
      "build/**",
      "coverage/**",
      "node_modules/**",
      "src-tauri/gen/**",
      "target/**",
    ],
  },
  js.configs.recommended,
  ts.configs.recommended,
  svelte.configs.recommended,
  svelte.configs.prettier,
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
  },
  {
    files: ["**/*.svelte", "**/*.svelte.ts", "**/*.svelte.js"],
    languageOptions: {
      parserOptions: {
        extraFileExtensions: [".svelte"],
        parser: ts.parser,
        svelteConfig,
      },
    },
  },
  {
    files: ["e2e/specs/**/*.mjs"],
    languageOptions: {
      globals: {
        ...globals.mocha,
        $: "readonly",
        $$: "readonly",
        browser: "readonly",
      },
    },
  },
);
