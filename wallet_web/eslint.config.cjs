const pluginVue = require("eslint-plugin-vue")
const { defineConfigWithVueTs, vueTsConfigs } = require("@vue/eslint-config-typescript")
const skipFormatting = require("@vue/eslint-config-prettier/skip-formatting")
const playwright = require("eslint-plugin-playwright")
const globals = require("globals")

module.exports = defineConfigWithVueTs(
    {
        name: "app/files-to-lint",
        files: ["**/*.{js,mjs,cjs,ts,mts,vue}"],
    },

    {
        name: "app/files-to-ignore",
        ignores: ["**/dist/**", "**/dist-ssr/**", "**/coverage/**"],
    },

    pluginVue.configs["flat/essential"],
    vueTsConfigs.recommended,
    skipFormatting,

    {
        files: ["e2e/**/*.{test,spec}.{js,ts,jsx,tsx}"],
        ...playwright.configs["flat/recommended"],
    },

    {
        languageOptions: {
            ecmaVersion: "latest",
            globals: {
                ...globals.node,
            },
        },
    },

    // Custom rules to handle the specific errors encountered when upgrading to 9.29.0
    {
        files: ["**/*.{ts,vue}"],
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

    // Special case for the config file itself to allow requires
    {
        files: ["eslint.config.cjs"],
        rules: {
            "@typescript-eslint/no-require-imports": "off",
        },
    },
)
