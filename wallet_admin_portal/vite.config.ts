import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueDevTools from 'vite-plugin-vue-devtools'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    vueDevTools(),
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  server: {
    port: 5173,
    strictPort: true,
    proxy: {
      // Proxy to the wallet_provider (started via scripts/start-devenv.sh) so the SPA and
      // wallet_provider share one origin. It serves TLS with a self-signed dev cert, hence `secure: false`.
      "/admin-portal": { target: "https://localhost:3000", secure: false },
    },
  },
})
