import pluginJs from "@eslint/js"
import tseslint from "typescript-eslint"

/** @type {import('eslint').Linter.Config[]} */
export default [
  { files: ["**/*.{js,mjs,cjs,ts}"] },
  { ignores: ["allure-report/**/*"] },
  pluginJs.configs.recommended,
  ...tseslint.configs.recommended,
]
