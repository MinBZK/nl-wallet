import vue from "@vitejs/plugin-vue"
import { fileURLToPath, URL } from "node:url"
import { resolve } from "path"
import { defineConfig } from "vite"
import dts from "vite-plugin-dts"

export default defineConfig({
  server: {
    port: 5175
  },
  define: {
    "process.env": process.env
  },
  plugins: [
    vue({ customElement: true }),
    dts({ tsconfigPath: "tsconfig.build.json", cleanVueFileName: true, rollupTypes: true })
  ],
  build: {
    copyPublicDir: false,
    lib: {
      entry: resolve(__dirname, "lib/main.ts"),
      name: "nl_wallet_web",
      fileName: "nl-wallet-web",
      formats: ["es", "umd", "iife"]
    },
    rollupOptions: {}
  },
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./lib", import.meta.url))
    }
  }
})
