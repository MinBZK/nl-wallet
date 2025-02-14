const { defineConfig } = require("@playwright/test")

export default defineConfig({
  testDir: "../tests/unauthenticated",

  reporter: [
    ["list"],
    ["junit", { outputFile: "../test-results-unauthenticated/results_unauthenticated.xml" }],
  ],
  projects: [
    {
      name: "chromium-desktop",
      use: {
        browserName: "chromium",
        viewport: { width: 1920, height: 1080 },
      },
    },
  ],
  use: {
    baseURL: "https://" + process.env.GBA_FETCH_FRONTEND_INTERNAL_HOSTNAME_ONT,
    screenshot: "only-on-failure",
    video: "off",
    trace: "off",
  },
})
