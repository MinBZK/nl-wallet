import pluginJs from "@eslint/js";
import playwright from 'eslint-plugin-playwright'

/** @type {import('eslint').Linter.Config[]} */
export default [
  playwright.configs['flat/recommended'],
  pluginJs.configs.recommended,
];
