import { defineConfig } from "vitest/config"

export default defineConfig({
  test: {
    include: ["static/__tests__/**/*.test.js"],
  },
})
