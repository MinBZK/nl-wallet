import { createHash } from "node:crypto"
import { readFileSync } from "node:fs"
import { stdout, exit } from "node:process"

const scriptFile = readFileSync("dist/nl-wallet-web.iife.js", { encoding: "utf8" })
// Same regex as used in wallet_core/demo/demo_utils/src/lib.rs
const match = new RegExp(String.raw`\[\["styles",\['([^']+)']]]`).exec(scriptFile)
if (match == null) {
  stdout.write("ERROR: Could not find style in file\n")
  exit(1)
}
const styleHash = createHash("sha256").update(match[1]).digest("base64")
stdout.write(`sha256-${styleHash}\n`)
