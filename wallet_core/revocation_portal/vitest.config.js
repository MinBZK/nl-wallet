import { defineConfig } from "vitest/config"

export default defineConfig({
  test: {
    coverage: {
      enabled: true,
      provider: "v8",
      reporter: ["text", "lcov"],
      reportsDirectory: "coverage",
    },
    reporters: [
      "default",
      ["junit", { suiteName: "wallet_web tests", outputFile: "test-results/junit.xml" }],
      ["allure-vitest/reporter", { resultsDir: "allure-results" }],
    ],
    include: ["static/__tests__/**/*.test.js"],
  },
})
