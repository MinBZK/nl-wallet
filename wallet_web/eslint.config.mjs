import pluginVue from "eslint-plugin-vue"
import { defineConfigWithVueTs, vueTsConfigs } from "@vue/eslint-config-typescript"
import skipFormatting from "@vue/eslint-config-prettier/skip-formatting"
import playwright from "eslint-plugin-playwright"
import globals from "globals"

export default defineConfigWithVueTs(
  {
    name: "app/ignore-list",
    ignores: ["**/dist/**", "**/dist-ssr/**", "**/coverage/**"],
  },

  pluginVue.configs["flat/essential"],
  vueTsConfigs.recommended,
  skipFormatting,

  {
    name: "app/main-config",
    files: ["**/*.{js,mjs,cjs,ts,mts,vue}"],
    languageOptions: {
      ecmaVersion: "latest",
      globals: {
        ...globals.node,
        ...globals.browser,
      },
    },
    rules: {
      "@typescript-eslint/no-explicit-any": "off",
      "@typescript-eslint/no-unused-expressions": "off",
      "@typescript-eslint/no-namespace": "off",
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
          caughtErrorsIgnorePattern: "^_",
        },
      ],
    },
  },

  {
    name: "app/playwright-tests",
    files: ["e2e/**/*.{test,spec}.{js,ts,jsx,tsx}"],
    ...playwright.configs["flat/recommended"],
  },
)
