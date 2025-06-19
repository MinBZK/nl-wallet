import process from "node:process"
import { defineConfig } from "@playwright/test"

export default defineConfig({
  testDir: "../tests/authenticated",
  reporter: [
    ["list"],
    ["junit", { outputFile: "../test-results-authenticated/results_authenticated.xml" }],
    ["allure-playwright"],
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
    clientCertificates: [
      {
        origin: process.env.GBA_FETCH_FRONTEND_URL,
        pfxPath: process.env.GBA_FETCH_FRONTEND_CLIENT_CERT_PATH,
        passphrase: process.env.GBA_FETCH_FRONTEND_CLIENT_CERT_PASSPHRASE,
      },
    ],
    baseURL: process.env.GBA_FETCH_FRONTEND_URL,
    screenshot: "only-on-failure",
    video: "off",
    trace: "off",
  },
})
