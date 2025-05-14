import vue from "@vitejs/plugin-vue"
import { fileURLToPath, URL } from "node:url"
import { resolve } from "path"
import { defineConfig, loadEnv } from "vite"
import dts from "vite-plugin-dts"

export default defineConfig(({ mode }) => {
  process.env = { ...process.env, ...loadEnv(mode, process.cwd()) }

  if (!process.env.VITE_HELP_BASE_URL) {
    throw new Error("VITE_HELP_BASE_URL is required")
  } else {
    new URL(process.env.VITE_HELP_BASE_URL) // throws if it's not a valid URL
  }

  return {
    server: {
      port: 5175,
    },
    define: {
      "process.env.NODE_ENV": `'${process.env.NODE_ENV}'`,
      __APP_VERSION__: JSON.stringify(process.env.npm_package_version),
    },
    plugins: [
      vue(),
      dts({ tsconfigPath: "tsconfig.build.json", cleanVueFileName: true, rollupTypes: true }),
    ],
    build: {
      copyPublicDir: false,
      lib: {
        entry: resolve(__dirname, "lib/main.ts"),
        name: "nl_wallet_web",
        fileName: "nl-wallet-web",
        formats: ["es", "umd", "iife"],
      },
    },
    resolve: {
      alias: {
        "@": fileURLToPath(new URL("./lib", import.meta.url)),
      },
    },
  }
})
