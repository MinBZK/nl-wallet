---
name: vuln-scan-diff
description: >-
  Static vulnerability scan scoped to a git commit range. Extracts changed
  files and diff hunks via `git diff`, reviews only the modified code in full
  file context, and writes VULN-FINDINGS-DIFF.json + .md for /triage to
  consume. Read-only — no building, running, or network. Use when asked to
  "scan this PR", "review changes between commits", "check what changed in
  <range>", or to focus vuln-scan on a branch diff.
argument-hint: "<repo-dir> <commit-range> [--focus <area>] [--exclude <path>] [--single] [--extra <file>] [--no-score]"
allowed-tools:
  - Read
  - Glob
  - Grep
  - Write
  - Task
  - Bash(git diff:*)
  - Bash(git log:*)
  - Bash(git show:*)
  - Bash(git rev-parse:*)
  - Bash(git -C:*)
  - Bash(rg:*)
  - Bash(grep:*)
  - Bash(ls:*)
  - Bash(wc:*)
  - Bash(head:*)
  - Bash(file:*)
---

# /vuln-scan-diff

Static vulnerability scan restricted to a **git commit range**. Reviews
only the files touched in the range, in the context of their full current
content, and produces `VULN-FINDINGS-DIFF.json` (plus `.md`) for `/triage`.

**No code execution.** Reads source and reasons about it only.

**Tool fallbacks.** Prefer built-in Glob and Grep. When unavailable, use the
Bash fallbacks in `allowed-tools`. Do not write helper scripts or pipe target
content into a shell interpreter.

## Arguments

- `<repo-dir>` (required) — git repository root.
- `<commit-range>` (required) — any range `git diff` accepts: `main..HEAD`,
  `abc123..def456`, `HEAD~5..HEAD`, a single SHA (expanded to `<sha>^..<sha>`),
  or a branch name.
- `--focus <area>` — restrict to this focus area (repeatable). Skips recon.
- `--exclude <path>` — path under `<repo-dir>` to skip (repeatable). Applied
  after the built-in non-source filter. Forwarded to subagents so they skip
  excluded paths when chasing context.
- `--single` — no subagent fan-out; one sequential pass.
- `--extra <file>` — append contents to the review brief.
- `--no-score` — skip the confidence calibration pass.

## Step 1 — Resolve the diff

1. Confirm `<repo-dir>` is a git repository:
   ```
   git -C <repo-dir> rev-parse --git-dir
   ```
   Stop with a clear error if not.

2. Normalise the commit range: if the user supplied a bare SHA, expand it to
   `<sha>^..<sha>`. Otherwise use the range as given.

3. List changed source files (excluding deletions):
   ```
   git -C <repo-dir> diff --name-only --diff-filter=ACMRT <range>
   ```
   Apply the built-in non-source filter — skip paths matching:
   `**/test*`, `**/fixture*`, `**/vendor*`, `**/generated*`, `**/docs/**`,
   `**/*.md`, `**/*.json`, `**/*.yaml`, `**/*.yml`, `**/*.lock`.

4. Apply `--exclude` filtering: normalise each path (strip leading `./` and
   any leading/trailing `/`). Drop files whose repo-relative path equals an
   excluded path or starts with `<excluded>/`. Warn but continue on paths that
   do not exist.

5. If no source files remain after filtering, tell the user and stop
   successfully (an all-docs or all-config diff is not an error).

6. Show the user:
   - Resolved range (both SHAs via `git rev-parse`)
   - One-line commit log: `git -C <repo-dir> log --oneline <range>`
   - Changed source files and their count
   - Any `--exclude` paths and how many files each removed

## Step 2 — Scope focus areas

1. Check for `<repo-dir>/THREAT_MODEL.md`. If present, parse section 3 (entry
   points) and section 4 (threats) to derive focus areas, same as `/vuln-scan`.

2. If no THREAT_MODEL.md and no `--focus`: group changed files by subsystem
   (top-level directory or logical module). Describe each group as:
   `<subsystem> (<N> files changed) — <key operations touched>`. Aim for
   3–8 groups. If all changes are in one subsystem, treat the whole diff as one
   focus area.

3. If `--focus` was given, filter changed files to those belonging to each
   area. Drop any area with no files after exclusions; stop if all are empty.

4. Report focus areas and file assignments before fanning out.

## Step 3 — Fan out

For each file in each focus area, fetch the diff and current content:

```
git -C <repo-dir> diff <range> -- <file>
Read <repo-dir>/<file>
```

Unless `--single`, spawn **one Task subagent per focus area** concurrently
(cap at 10). For diffs touching 3 or fewer source files, use `--single`
automatically.

### Review brief (per subagent)

```
You are conducting authorized static security review of a NL Wallet code
change. Focus area: **{focus_area}**. Other agents cover other areas.

REPOSITORY: {repo_dir}
COMMIT RANGE: {range} ({base_sha}..{head_sha})
TRUST BOUNDARY: {from THREAT_MODEL.md section 3, or "untrusted input → process memory"}
EXCLUDED PATHS — do not read or report from these:
{normalised --exclude paths, or "none"}

CHANGED FILES IN YOUR FOCUS AREA:
{list of file paths}

If a path you would read for context falls under an excluded path, skip it
silently.

FOR EACH FILE:
  1. The git diff hunks (lines prefixed + or -)
  2. The full current file for surrounding context

DIFF HUNKS:
{for each file: "--- <file>\n" + raw diff output}

FULL FILE CONTENTS:
{for each file: "=== <file> ===\n" + full content}

TASK: identify candidate vulnerabilities introduced or worsened by this diff.
Focus on:
  - NEW code (+ lines) that introduces a vulnerability
  - MODIFIED code that weakens an existing defence
  - CONTEXT lines revealing a pre-existing bug now reachable via new paths

Static review only — do NOT build, run, or probe anything.

REPORTING BAR: report anything with a plausible exploit path. If unsure,
include the finding with a low confidence score.

WHAT TO LOOK FOR

  MEMORY SAFETY (C/C++ and unsafe/FFI code) — HIGH VALUE:
  - Buffer overflows (heap, stack, global)
  - Use-after-free, double-free
  - Integer overflow feeding an allocation or array index
  - Format-string bugs
  - Unbounded recursion or allocation driven by untrusted size fields

  INJECTION AND CODE EXECUTION — HIGH VALUE:
  - SQL, command, LDAP, XPath, NoSQL, template injection
  - Path traversal in file operations
  - Unsafe deserialization, eval/exec injection
  - XSS (reflected, stored, DOM-based) — see exclusion note below

  AUTH, CRYPTO, AND DATA — HIGH VALUE:
  - Authentication or authorisation bypass, privilege escalation
  - TOCTOU on a security check
  - Hardcoded secrets, weak crypto, broken cert validation
  - Sensitive data in logs or error responses

  LOW VALUE — note briefly, keep searching:
  - Null-pointer deref at small fixed offsets with no attacker control
  - Assertion failures and clean error returns

DO NOT REPORT:
  - Volumetric DoS / rate-limiting / resource exhaustion — BUT unbounded
    recursion, algorithmic complexity blowup, and ReDoS on untrusted input
    ARE reportable
  - Memory-safety issues in memory-safe languages outside unsafe/FFI
  - XSS in React, Angular, or Vue unless via a raw-HTML escape hatch
  - Findings in test files, fixtures, build scripts, docs, or notebooks
  - Missing hardening with no concrete exploit path
  - Env vars and CLI flags as the attack vector
  - Regex injection, log spoofing, open redirect, missing audit logs
  - Outdated third-party dependency versions
  - Pre-existing bugs that this diff neither touches nor makes newly reachable

{if --extra was given: append its contents here}

Trace each finding: where untrusted input enters, what path reaches the
sink, and whether the diff introduced or exposed that path.

OUTPUT — one XML block per finding, nothing else:

<finding>
<id>F-{focus_idx:02d}-{n:02d}</id>
<file>{relative/path}</file>
<line>{line_number_in_current_file}</line>
<diff_introduced>{yes | pre-existing-now-reachable}</diff_introduced>
<category>{heap-buffer-overflow | use-after-free | integer-overflow | sql-injection | command-injection | path-traversal | deserialization | xss | auth-bypass | hardcoded-secret | ...}</category>
<severity>{HIGH | MEDIUM | LOW}</severity>
<confidence>{0.0-1.0}</confidence>
<title>{one line}</title>
<description>{root cause, attacker control, trigger, data flow — cite line numbers and note new vs changed lines}</description>
<exploit_scenario>{concrete attack: what input, from where, causing what outcome}</exploit_scenario>
<recommendation>{specific fix}</recommendation>
</finding>

SEVERITY: HIGH = directly exploitable → RCE, data breach, auth bypass.
MEDIUM = significant impact under specific conditions. LOW = defence-in-depth.

If nothing reportable, emit one <finding> with category=none and a brief
note of what was covered.
```

## Step 4 — Collate

1. Collect all `<finding>` blocks. Drop `category=none` placeholders.
2. **Light dedupe**: same `file:line` + same `category` → keep the longer
   description, record the duplicate id.
3. Assign stable ids `F-001`, `F-002`, … ordered by (severity desc, file, line).

## Step 4b — Confidence calibration (skip with `--no-score`)

Same logic as `/vuln-scan` Step 3b. Spawn one Task per finding concurrently.
Nothing is dropped — scores are recalibrated only.

### Scoring brief

```
Score ONE security finding independently. You are NOT deciding whether to
keep it. Decide how likely it is to survive triage.

FINDING:
{full <finding> block}

REPOSITORY: {repo_dir} (Read/Grep only; do NOT execute)
DIFF CONTEXT: {the diff hunk for the cited file}

Step 1: Read {file} at {line}. Does the code match the description?
Step 2: Check for false-positive patterns: volumetric DoS, memory-safe
  language, test/doc file, framework auto-escaping, operator input, pre-
  existing issue not touched by this diff.
Step 3: Score 1–10:
  1–3  likely false positive
  4–5  plausible but speculative
  6–7  credible, worth investigating
  8–10 high confidence, clear pattern

OUTPUT (exactly this format):
  CONFIDENCE: <1-10>
  REASON: <one line>
```

After all votes return: normalise confidence to 0.0–1.0, attach
`confidence_reason`. Re-sort by (confidence desc, severity desc, file, line)
and reassign ids `F-001..`. Track `low_confidence_count` (confidence < 0.4).

## Step 5 — Write output

Write both files to `<repo-dir>/`:

**`VULN-FINDINGS-DIFF.json`**

```json
{
  "target": "<repo-dir>",
  "commit_range": "<range>",
  "base_sha": "<sha>",
  "head_sha": "<sha>",
  "scanned_at": "<iso8601>",
  "changed_files_scanned": ["..."],
  "excluded_paths": ["..."],
  "focus_areas": ["..."],
  "findings": [
    {
      "id": "F-001",
      "file": "relative/path.rs",
      "line": 123,
      "diff_introduced": "yes",
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

**`VULN-FINDINGS-DIFF.md`** — summary table (id | severity | category |
file:line | diff_introduced | title), then one `### F-NNN` section per finding.

## Step 6 — Hand back

Report to the user:

1. Resolved range: `<base_sha>..<head_sha>` with commit summary.
2. Counts: N findings (H/M/L split, X low-confidence), across K focus areas,
   from M changed source files. Name excluded paths and files skipped.
3. Top 3 by confidence, one line each.
4. Suggested next step: `/triage <repo-dir>/VULN-FINDINGS-DIFF.json --repo <repo-dir>`
5. Reminder: findings are scoped to this diff; pre-existing bugs outside the
   diff are intentionally out of scope — use `/vuln-scan` for a full-tree scan.

## Constraints

- **Never execute target code.** No Bash beyond the whitelisted git and search
  commands.
- **No fabricated line numbers.** Every `file:line` must come from something
  you actually Read or Grep'd.
- **Stay inside `<repo-dir>`.** Do not follow symlinks or `..` paths out.
- **Scope to the diff.** Report pre-existing bugs only when the diff creates
  a new call path that makes them reachable.
- Findings are input to `/triage`, not final verdicts. This skill never drops
  a finding; Step 4b only re-ranks.
