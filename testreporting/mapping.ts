import { basename, extname } from "node:path"

// Mapping to rename filenames to human-readable strings
const filenameToGroupMapping: Record<string, string> = {
  "browsertest-wallet-web": "Browsertests",
  "browsertest-fallback-pages": "Browsertests",
  "browsertest-revocation-portal": "Browsertests",
  "e2e-android": "End-to-end tests (Android)",
  "e2e-ios": "End-to-end tests (iOS)",
  flutter: "Wallet App",
  "flutter-ui": "Golden Tests",
  web: "Wallet Web",
  ios: "Platform Support iOS",
  android: "Platform Support Android",
  rust: "Wallet Core",
}

export function fileNameToGroup(fileName: string) {
  return filenameToGroupMapping[basename(fileName, extname(fileName))] || fileName
}
