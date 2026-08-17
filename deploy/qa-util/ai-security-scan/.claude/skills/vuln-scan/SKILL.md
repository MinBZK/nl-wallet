---
name: vuln-scan
description: >-
  Static source-code vulnerability scan. Reads a target directory (and
  THREAT_MODEL.md if present), spawns parallel review subagents per focus
  area, and writes VULN-FINDINGS.json + .md for /triage to consume. Read-only
  — no building, running, or network. Use when asked to "scan for vulns",
  "review this code for security issues", "find bugs in <dir>", or as the
  step between /threat-model and /triage.
argument-hint: "<target-dir> [--focus <area>] [--exclude <path>] [--single] [--extra <file>] [--no-score]"
allowed-tools:
  - Read
  - Glob
  - Grep
  - Write
  - Task
  - Bash(rg:*)
  - Bash(grep:*)
  - Bash(ls:*)
  - Bash(wc:*)
  - Bash(head:*)
  - Bash(file:*)
---

# /vuln-scan

Static vulnerability scan of a source tree. Produces `VULN-FINDINGS.json`
(and a human-readable `VULN-FINDINGS.md`) for `/triage` to ingest.

**No code execution.** This skill reads source and reasons about it. PoC
reproduction or build-time verification is out of scope; that belongs in a
controlled manual test by a human.

**Tool fallbacks.** Prefer the built-in Glob and Grep tools. When they are
unavailable, use the Bash fallbacks listed in `allowed-tools`: `rg --files`
or `ls -R` to enumerate, `rg -n` or `grep -rn` to search, `wc`/`head`/`file`
to inspect. Do not write helper scripts or pipe target content into a shell
interpreter.

## Arguments

- `<target-dir>` (required) — directory to scan.
- `--focus <area>` — restrict to this focus area (repeatable). Skips recon.
- `--exclude <path>` — path under `<target-dir>` to skip entirely (repeatable).
  Matched as a path prefix relative to `<target-dir>`. Leading/trailing slashes
  are stripped. Excluded paths are not enumerated, not assigned to any focus
  area, and the list is forwarded to every subagent so they refuse to read inside
  them.
- `--single` — skip subagent fan-out; run one sequential pass. Useful for tiny
  targets or debugging.
- `--extra <file>` — append the contents of `<file>` to the review brief. Use
  for project-specific vulnerability classes or compliance checks.
- `--no-score` — skip the confidence-calibration pass (Step 3b). Findings
  retain the scanner's self-reported score only.

## Step 1 — Scope

1. Resolve `<target-dir>`. If it does not exist or contains no source files,
   stop with a clear error.
2. Normalise `--exclude` paths: strip leading `./` and any leading/trailing
   `/`. Warn (but continue) if an excluded path does not exist. A source file
   at `<target-dir>/<rel>` is excluded if `<rel>` equals an excluded path or
   begins with `<excluded>/`. Apply this filter every time files are enumerated.
3. Check for `<target-dir>/THREAT_MODEL.md`. If present, read its section 3
   (entry points & trust boundaries) and section 4 (threats) to derive focus
   areas and threat classes.
4. If no THREAT_MODEL.md and no `--focus`: run a quick recon — list the source
   tree (with exclusions applied), read entry points and dispatch code, then
   propose 3–10 focus areas in the form `<subsystem> (<file/function>) — <key
   operations>`.
5. If `--focus` was given, use exactly those areas. Drop any area whose files
   all fall under an `--exclude` path; if every area is fully excluded, stop.

Report the chosen focus areas, source-file count, and any excluded paths before
fanning out.

## Step 2 — Fan out

Unless `--single`, spawn **one Task subagent per focus area** concurrently.
Cap at 10 parallel tasks. For targets with fewer than 15 source files, fall
through to `--single` automatically.

Each subagent receives the review brief below with its focus area filled in.

### Review brief (per subagent)

```
You are conducting authorized static security review of NL Wallet source
code. Focus area: **{focus_area}**. Other agents cover other areas.

TARGET: {target_dir}
TRUST BOUNDARY: {from THREAT_MODEL.md section 3, or "untrusted input → process memory"}
EXCLUDED PATHS — do not read or report from these:
{normalised --exclude paths, or "none"}

Read the source in your focus area and identify candidate vulnerabilities.
Static review only — do NOT build, run, or probe anything. Skip excluded
paths silently.

REPORTING BAR: report anything with a plausible exploit path. If unsure,
include the finding with a low confidence score — downstream triage handles
verification. Skip style issues, best-practice gaps with no exploit, and
purely theoretical problems.

WHAT TO LOOK FOR

  MEMORY SAFETY (C/C++ and unsafe/FFI code) — HIGH VALUE:
  - Buffer overflows (heap, stack, global)
  - Use-after-free, double-free
  - Integer overflow that feeds an allocation or array index
  - Format-string bugs
  - Unbounded recursion or allocation driven by untrusted size fields

  INJECTION AND CODE EXECUTION — HIGH VALUE:
  - SQL, command, LDAP, XPath, NoSQL, template injection
  - Path traversal in file operations
  - Unsafe deserialization (pickle, YAML load, native), eval/exec injection
  - XSS (reflected, stored, DOM-based) — see exclusion note below

  AUTH, CRYPTO, AND DATA — HIGH VALUE:
  - Authentication or authorisation bypass, privilege escalation
  - TOCTOU on a security check
  - Hardcoded secrets, weak or broken cryptography, missing cert validation
  - Sensitive data (credentials, PII) written to logs or error responses

  LOW VALUE — note briefly, keep searching:
  - Null-pointer deref at small fixed offsets with no attacker control
  - Assertion failures and clean error returns

DO NOT REPORT:
  - Volumetric DoS / rate-limiting / resource exhaustion — BUT unbounded
    recursion, algorithmic-complexity blowup, and ReDoS on untrusted input
    ARE reportable
  - Memory-safety issues in memory-safe languages outside unsafe/FFI
  - XSS in React, Angular, or Vue unless via a raw-HTML escape hatch
    (dangerouslySetInnerHTML, bypassSecurityTrustHtml, v-html)
  - Findings in test files, fixtures, build scripts, docs, or notebooks
  - Missing hardening with no concrete exploit path
  - Env vars and CLI flags as the attack vector (operator-controlled inputs)
  - Regex injection, log spoofing, open redirect, missing audit logs
  - Outdated third-party dependency versions

{if --extra was given: append its contents here}

For every finding you report, trace: where untrusted input enters, what path
reaches the sink, and what condition triggers the vulnerability.

OUTPUT — one XML block per finding, nothing else:

<finding>
<id>F-{focus_idx:02d}-{n:02d}</id>
<file>{relative/path}</file>
<line>{line_number}</line>
<category>{heap-buffer-overflow | use-after-free | integer-overflow | sql-injection | command-injection | path-traversal | deserialization | xss | auth-bypass | hardcoded-secret | ...}</category>
<severity>{HIGH | MEDIUM | LOW}</severity>
<confidence>{0.0-1.0}</confidence>
<title>{one line}</title>
<description>{root cause, attacker control, trigger condition, data flow from entry to sink — cite line numbers}</description>
<exploit_scenario>{concrete attack: what input, from where, causing what outcome}</exploit_scenario>
<recommendation>{specific fix}</recommendation>
</finding>

SEVERITY: HIGH = directly exploitable → RCE, data breach, auth bypass.
MEDIUM = significant impact under specific conditions. LOW = defence-in-depth.

If nothing reportable is found after a thorough review, emit one <finding>
with category=none and a brief note of what was covered.
```

## Step 3 — Collate

1. Collect all `<finding>` blocks. Drop `category=none` placeholders.
2. **Light dedupe**: if two findings share the same `file:line` and `category`,
   keep the one with the longer description and record the other's id as a
   duplicate. Heavy deduplication is for `/triage`.
3. Assign stable ids `F-001`, `F-002`, … ordered by (severity desc, file, line).

## Step 3b — Confidence calibration (skip with `--no-score`)

A lightweight second-opinion pass that adjusts `confidence` scores so
high-signal findings surface first. **No findings are dropped** at this stage —
the pass only re-ranks. Spawn one Task subagent per finding concurrently.

### Scoring brief

```
Score ONE security finding independently. You are NOT deciding whether to
keep it — every finding is kept. Decide how likely it is to survive rigorous
triage.

FINDING:
{full <finding> block}

TARGET: {target_dir} (Read/Grep only; do NOT execute)

Step 1: Read the cited code. Does it do what the description claims?
Step 2: Check for common false-positive patterns: volumetric DoS,
  memory-safe language, test/fixture/doc file, framework auto-escaping,
  operator-controlled input, missing-hardening-only, outdated dep.
Step 3: Score 1–10 for how real and actionable this finding is:
  1–3  likely false positive
  4–5  plausible but speculative
  6–7  credible, worth investigating
  8–10 high confidence, clear pattern

OUTPUT (exactly this format, nothing else):
  CONFIDENCE: <1-10>
  REASON: <one line>
```

After all votes return: normalise confidence to 0.0–1.0, attach
`confidence_reason`. Re-sort findings by (confidence desc, severity desc,
file, line) and reassign ids `F-001..`. Track `low_confidence_count`
(confidence < 0.4) for the summary.

## Step 4 — Write output

Write both files to `<target-dir>/`:

**`VULN-FINDINGS.json`**

```json
{
  "target": "<target-dir>",
  "scanned_at": "<iso8601>",
  "focus_areas": ["..."],
  "excluded_paths": ["..."],
  "findings": [
    {
      "id": "F-001",
      "file": "relative/path.rs",
      "line": 123,
      "category": "integer-overflow",
      "severity": "HIGH",
      "confidence": 0.9,
      "title": "...",
      "description": "...",
      "exploit_scenario": "...",
      "recommendation": "...",
      "confidence_reason": "..."
    }
  ],
  "summary": {"total": 0, "high": 0, "medium": 0, "low": 0, "low_confidence": 0}
}
```

**`VULN-FINDINGS.md`** — a summary table (id | severity | category | file:line
| title) followed by one `### F-NNN` section per finding with the full detail.

## Step 5 — Hand back

Report to the user:

1. Counts: N findings (H/M/L split, X low-confidence), across K focus areas,
   from M source files. Name any excluded paths and the files they skipped.
2. Top 3 by confidence, one line each.
3. Suggested next step: `/triage <target-dir>/VULN-FINDINGS.json --repo <target-dir>`
4. Reminder: findings are static candidates, not verified.

## Constraints

- **Never execute target code.** If asked to reproduce a finding or write a
  PoC, decline and suggest a controlled human test.
- **No fabricated line numbers.** Every `file:line` must come from something
  you actually Read or Grep'd. If the exact line is uncertain, cite the
  function and say so.
- **Stay inside `<target-dir>`.** Do not follow symlinks or `..` paths out.
- Findings are input to `/triage`, not final verdicts. This skill never drops
  a finding; the confidence pass only re-ranks.
