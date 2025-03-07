// Import from plugin-api to not init cli
import { defineConfig } from "@allurereport/plugin-api"

export default defineConfig({
  name: "NL-Wallet",
  plugins: {
    awesome: {
      options: {
        singleFile: true,
      },
    },
  },
})
