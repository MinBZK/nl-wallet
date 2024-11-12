import { fileURLToPath } from "node:url"
import { configDefaults, defineConfig, mergeConfig } from "vitest/config"
import viteConfig from "./vite.config"

export default mergeConfig(
  viteConfig({ mode: "test", command: "serve" }),
  defineConfig({
    test: {
      environment: "happy-dom",
      exclude: [...configDefaults.exclude, "e2e/**"],
      root: fileURLToPath(new URL("./", import.meta.url)),
      coverage: {
        reporter: ["text", "lcov"],
      },
      reporters: [
        "default",
        ["junit", { suiteName: "wallet_web tests", outputFile: "coverage/tests.xml" }],
      ],
    },
  }),
)
