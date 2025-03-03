import process from "node:process"
import { defineConfig } from "@playwright/test"

export default defineConfig({
  testDir: "../tests",
  reporter: [["list"], ["junit", { outputFile: "../test-results/results.xml" }]],
  projects: [
    {
      name: "chromium-desktop",
      use: {
        browserName: "chromium",
        viewport: { width: 1920, height: 1080 },
        userAgent:
          "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
      },
    },
  ],
  use: {
    baseURL: process.env.UNIVERSAL_LINK_BASE,
    screenshot: "only-on-failure",
    video: "retain-on-failure",
    trace: "retain-on-failure",
  },
})
