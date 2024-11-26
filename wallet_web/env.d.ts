/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_HELP_BASE_URL: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}

declare const __APP_VERSION__: string
