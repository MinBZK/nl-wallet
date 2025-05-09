import { basename, extname } from "node:path"

// Mapping to rename filenames to human-readable strings
const filenameToGroupMapping: Record<string, string> = {
  "browsertest-demo-rp": "Browsertests",
  "browsertest-gbafetch": "Browsertests",
  "browsertest-fallback-pages": "Browsertests",
  e2e: "End-to-end tests",
  flutter: "Wallet App",
  "flutter-ui": "Golden Tests",
  web: "Wallet Web",
  ios: "Platform Support iOS",
  android: "Platform Support Android",
  rust: "Wallet Core",
  "rust-gba-pid": "GBA PID tests",
}

export function fileNameToGroup(fileName: string) {
  return filenameToGroupMapping[basename(fileName, extname(fileName))] || fileName
}
