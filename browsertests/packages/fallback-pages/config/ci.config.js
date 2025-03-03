import process from "node:process"
import { defineConfig } from "@playwright/test"

const screenSizes = [
  { name: "desktop", width: 1920, height: 1080 },
  { name: "tablet", width: 768, height: 1024 },
  { name: "mobile", width: 375, height: 667 },
]

const browsers = ["chromium", "webkit"]

const projects = []

browsers.forEach((browserName) => {
  screenSizes.forEach((screen) => {
    let userAgent

    if (screen.name === "mobile") {
      userAgent =
        "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.0 Mobile/15E148 Safari/604.1"
    } else if (screen.name === "tablet") {
      userAgent =
        "Mozilla/5.0 (iPad; CPU OS 14_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"
    } else {
      userAgent =
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"
    }

    projects.push({
      name: `${browserName}-${screen.name}`,
      use: {
        browserName,
        viewport: { width: screen.width, height: screen.height },
        userAgent,
      },
    })
  })
})

export default defineConfig({
  testDir: "../tests",
  reporter: [
    ["list"],
    ["junit", { outputFile: "../test-results/results.xml" }],
    ["allure-playwright"],
  ],
  projects,
  use: {
    baseURL: process.env.UNIVERSAL_LINK_BASE,
    screenshot: "only-on-failure",
    video: "retain-on-failure",
    trace: "retain-on-failure",
  },
})
