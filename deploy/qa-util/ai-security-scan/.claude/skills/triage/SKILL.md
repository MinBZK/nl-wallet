---
name: triage
description: Triage a batch of raw security findings. Verify each is real,
  collapse duplicates, re-rank by derived exploitability, and tag with an
  owner. Takes a directory or file of scanner output and writes TRIAGE.json
  + TRIAGE.md sorted by what actually needs engineering attention. Use when
  asked to "triage findings", "validate scanner output", "prioritize vulns",
  or "review the backlog". Runs interactively by default; pass --auto to
  skip the interview.
argument-hint: "<findings-path> [--auto] [--votes N] [--repo PATH] [--fp-rules FILE] [--fresh]"
allowed-tools:
  - Read
  - Glob
  - Grep
  - Write
  - Task
  - AskUserQuestion
  - Bash(git log:*)
  - Bash(git -C:*)
  - Bash(jq:*)
  - Bash(find:*)
  - Bash(ls:*)
  - Bash(wc:*)
  - Bash(python3 .claude/skills/_lib/checkpoint.py:*)
---

# /triage

Adversarial triage of raw scanner output. Four jobs: **verify** each finding
is real, **deduplicate** across runs and scanners, **rank** survivors by
exploitability (not the scanner's claimed severity), and **route** each to a
component owner. The output is a short, ranked, owned list ready for
engineering attention.

**Arguments** (parse from `$ARGUMENTS`; do not rely on positional expansion):

- First positional (required): path to a findings file or directory.
- `--auto` — skip the interactive interview; use built-in defaults.
- `--votes N` — verifier votes per finding (default 3; use 1 for speed, 5 for
  high-stakes batches).
- `--repo PATH` — path to the target codebase for source access (default cwd).
  Verification stops with an error if cited files are not reachable.
- `--fp-rules FILE` — append the contents of FILE to the verifier exclusion
  list (Phase 3a). One rule per line or paragraph.
- `--fresh` — ignore any existing checkpoint in `./.triage-state/` and start
  from Phase 0.

**Do not execute target code.** Every conclusion must come from reading
source. This applies to the orchestrator and every subagent; include this
constraint in every Task prompt.

**No network access.** No package-registry lookups, CVE-database queries, or
upstream-commit fetches.

---

## Checkpointing

Large batches can exhaust context mid-way, particularly Phase 3 which spawns
`candidates × votes` verifiers. Phase state is persisted to `./.triage-state/`
so a new session can resume from the last completed phase.

All checkpoint I/O goes through `python3 .claude/skills/_lib/checkpoint.py`
(atomic writes, JSON-validated). Never write `progress.json` directly with the
Write tool. Never pass payloads via heredoc or stdin — write to `_chunk.tmp`
first, then call checkpoint with `--from`.

State files in `./.triage-state/`:
- `progress.json` — sole source of truth for resume position:
  `{"status": "running"|"complete", "phase_done": N, "shards_done": [...]}`
- `phaseN.json` — payload for phase N
- `_chunk.tmp` — transient buffer; overwritten before every checkpoint call

**At the start of every run:**
```
python3 .claude/skills/_lib/checkpoint.py load ./.triage-state
```
- `status == "absent"`, `"complete"`, or `--fresh` in `$ARGUMENTS` →
  fresh start: reset the state dir, then go to Phase 0.
- `status == "running"` with `phase_done == N` → resume: read
  `phase0.json`…`phaseN.json` in order (and any shard files listed in
  `shards_done`), merging keys into working state. Print
  `Resuming from checkpoint: Phase N complete`, then jump to Phase N+1.

**At the end of every phase N:**
1. Write tool → `./.triage-state/_chunk.tmp` with the phase output JSON.
2. Bash → `python3 .claude/skills/_lib/checkpoint.py save ./.triage-state <N> <name> --from ./.triage-state/_chunk.tmp`

**At the end of the run:**
```
python3 .claude/skills/_lib/checkpoint.py done ./.triage-state 6
```

---

## Phase 0: Mode selection and interview

### 0a. Parse arguments

Extract from `$ARGUMENTS`: findings path (first positional), `--auto`,
`--votes N` (default 3), `--repo PATH` (default `.`), `--fp-rules FILE`.
If no findings path, ask and stop. If `--fp-rules` is given, read the file
now and carry its contents as `context.extra_fp_rules` for injection into
the Phase 3a verifier prompt.

### 0b. Interactive interview (default)

Unless `--auto`, use **AskUserQuestion** to gather context. Batch into one
or two calls of up to four questions. Free-text answers are expected via
"Other"; the options are prompts, not constraints.

**Round 1** (single AskUserQuestion call):

1. **Environment** (single-select): `What kind of system produced these
   findings, and where does untrusted input enter?`
   Options: `Internet-facing web service (HTTP is untrusted)`,
   `Internal service (callers are authenticated peers)`,
   `Library / SDK (caller is the trust boundary)`,
   `CLI / batch tool (operator inputs trusted, file inputs not)`,
   `Embedded / firmware (physical access is in scope)`.

2. **Threat model** (multi-select): `What must never happen in this system?
   Free text is best.`
   Options: `Unauthenticated remote code execution`,
   `Tenant-to-tenant data leakage`, `Privilege escalation to admin`,
   `Supply-chain compromise of downstream users`,
   `Denial of service against a paid SLA`,
   `Compliance-scoped data exposure (PII / PCI / PHI)`.

3. **Scoring** (single-select): `How should severity be expressed?`
   Options: `Derived HIGH/MEDIUM/LOW from preconditions (default)`,
   `CVSS v3.1 vector + base score`, `CVSS v4.0 vector + base score`,
   `OWASP Risk Rating (likelihood × impact)`,
   `Organisation bug-bar (describe in Other)`.

4. **Noise tolerance** (single-select): `When verifiers disagree, which way
   should ties break?`
   Options: `Precision: drop anything not majority-confirmed (fewer FPs)`,
   `Recall: keep split votes as needs_manual_test (fewer misses)`,
   `Ask me per-finding when it happens`.

**Round 2** (conditional): if the threat-model answer was empty or generic,
or scoring was `Organisation bug-bar`, ask one targeted follow-up.

Record all answers as `context`, carried through every phase and echoed in
the output under `triage_context`.

### 0c. Auto-mode defaults

When `--auto` is set, skip AskUserQuestion and use:
- Environment: `Unknown — treat any externally-reachable entry point as
  untrusted; flag trust-boundary assumptions in rationale.`
- Threat model: empty (no boost).
- Scoring: derived HIGH/MEDIUM/LOW.
- Noise tolerance: precision.

**Checkpoint payload:**
```json
{"phase": 0, "context": {mode, environment, threat_model, scoring, noise_tolerance, votes_per_finding, repo, findings_path}}
```

---

## Phase 1: Ingest and normalise

### 1a. Detect input format

Inspect the findings path:

- **Directory**: glob for `**/*.json` and `**/*.jsonl`. Recognised containers
  in priority order:
  - `VULN-FINDINGS.json` (`{findings: [...]}` container): read `.findings[]`.
  - `reports/bug_*/report.json` or `reports/manifest.jsonl`: map
    `crash.crash_type` → `category`, `verdict.severity_rating` → `severity`,
    `report` prose → `description`, top ASAN frame → `file`/`line`.
  - `found_bugs.jsonl`: one finding per line.
  - Any other `*.json` whose root is a list or has a `findings`/`results`/
    `issues`/`vulnerabilities` array: extract that array.
- **Single `.json` / `.jsonl` file**: same recognition as above.
- **Markdown or text**: split on level-2/3 headings or `---` rules; extract
  `file`, `line`, `category`, `severity`, `description` from label patterns
  (`File:`, `Line:`, `Severity:`, `path:NN`). Mark as `markdown_heuristic`.

Stop and report what was found if nothing is parseable.

### 1b. Normalise fields

Build a canonical finding dict for each raw record. Pull what is present;
never guess what is absent.

| Canonical | Also accept |
|---|---|
| `file` | `path`, `location.file`, `filename`, ASAN top-frame file |
| `line` | `line_number`, `location.line`, `lineno` |
| `category` | `type`, `cwe`, `rule_id`, `crash_type`, `vulnerability_class` |
| `severity` | `severity_rating`, `level`, `priority`, `risk` |
| `title` | `name`, `summary`, `message` |
| `description` | `details`, `report`, `body`, `evidence` |
| `exploit_scenario` | `attack_scenario`, `poc`, `reproduction` |
| `preconditions` | `requirements`, `assumptions` |
| `recommendation` | `fix`, `remediation`, `mitigation` |
| `scanner_confidence` | `confidence`, `score`, `certainty` (normalise to 0.0–1.0) |

Attach to every finding:
- `id`: `f001`, `f002`, … in ingest order. If `scanner_confidence` is present
  on most findings, order by it descending so high-signal findings get verified
  first.
- `source`: relative path of the originating file plus format string.
- `missing_fields`: canonical fields that were absent.

**Unlocatable findings**: if `file` is missing or does not resolve under
`--repo`, mark immediately with `verdict: false_positive`,
`verify_verdict: needs_manual_test`, `confidence: 0`,
`refute_reasons: ["doesnt_exist"]`, and a rationale explaining no static
verification was possible. These skip dedup and verification and never absorb
or are absorbed by another finding.

### 1c. Locate the codebase

Resolve `--repo`. For the first 5 findings with a `file`, try: (a) `repo/file`
as given; (b) `file` as absolute or cwd-relative; (c) `repo/file` with common
prefixes stripped (`src/`, `app/`, `./`, repo basename). Record which strategy
worked and apply it globally. If no strategy resolves any files, stop and ask
the user for a `--repo` value.

**Checkpoint payload:**
```json
{"phase": 1, "context": {...}, "findings": [{normalised dicts}], "path_resolution": "<strategy>"}
```

---

## Phase 2: Deduplicate

Collapse duplicates before verification so the same finding does not burn N
verifier slots.

### 2a. Deterministic pass (inline, no subagent)

Cluster findings where all of:
- same `file` (after path normalisation), AND
- same `category` (case-insensitive, punctuation stripped), AND
- `line` numbers within 10 of each other (both-missing matches; one-side-
  missing does NOT).

Within each cluster, the canonical is the finding with fewest `missing_fields`
(ties break to lowest `id`). All others get `verdict: duplicate`,
`duplicate_of: <canonical id>`, and are removed from the working set.

### 2b. Semantic pass (one subagent, only if >1 cluster survives)

Spawn one Task with:

```
You are deduplicating security findings before expensive verification.
Two findings are DUPLICATES if fixing one would also fix the other.
Two findings are DISTINCT if they have independent root causes, even if they
share a category or file.

DUPLICATE:
- Same root cause described by different scanners or in different words
- A shared vulnerable helper reported once per call site
- A missing global control (auth check, output encoding) reported once per
  endpoint that lacks it
- A cause and its direct consequence in the same code path

DISTINCT:
- Different categories in the same file region
- Same file, same category, but different tainted variables reaching different sinks
- Same helper, two independent bugs inside it
- Two endpoints missing the same per-endpoint fix

Candidates (id | file:line | category | title, one per line):
{list}

Respond ONLY with lines of the form:
  GROUP: <canonical_id> <- <dup_id>, <dup_id>, ...

One line per group with duplicates. Omit singletons. No prose.
```

Apply GROUP lines: mark dup ids with `verdict: duplicate` and
`duplicate_of: <canonical>`, add to canonical's `absorbed`, remove from
working set.

Carry forward `candidates[]` = surviving canonicals.

**Checkpoint payload:**
```json
{"phase": 2, "context": {...}, "findings": [{all findings with verdicts}], "candidates": ["f001", ...]}
```

---

## Phase 3: Verify

For each candidate, N independent verifiers re-derive the claim from the
source and vote. Each starts from the code, not the scanner's description.
Verifiers never see each other's reasoning.

### 3a. Verifier prompt (assembled once, reused for every spawn)

```
You are a sceptical security engineer adversarially verifying ONE finding
from an automated scanner. Your default assumption is that the scanner is
WRONG. Re-derive the claim from the source code yourself.

Read-only access to: {REPO_PATH}
Read, Glob, and Grep only — stay inside {REPO_PATH}. Do NOT read, grep, or
glob outside that root. Do not build, run, install, or reach the network.

ENVIRONMENT: {context.environment or "Unknown. Treat any externally-reachable
entry point as untrusted."}

────────────────────────────────────────────────────────────────────────
PROCEDURE — follow all four steps:

1. READ THE CODE AT THE CITED LOCATION YOURSELF.
   Open {file} at line {line}. Understand what the code does. Do not trust
   the scanner's description.

2. TRACE REACHABILITY BACKWARDS FROM THE SINK.
   Grep for callers. Follow imports. Establish whether attacker-controlled
   input per the ENVIRONMENT can reach this line. For the first link in the
   call chain, READ the actual call site and QUOTE the file:line in your
   rationale. Unreachable code is the largest single source of false positives.

3. HUNT FOR PROTECTIONS.
   Actively look for reasons the finding is WRONG:
   - Input validation / sanitisation upstream of the sink
   - Framework auto-escaping, parameterised queries, prepared statements
   - Type constraints (value is an int, enum, fixed-length token)
   - Auth / authorisation gates before this path
   - Configuration that limits exposure (feature flag, debug-only, dead code)

4. STRESS-TEST EACH PROTECTION.
   For each protection found: does it apply on EVERY path to the sink, or
   only the one the scanner traced? Are there encodings, edge cases, or
   alternate entry points that bypass it?

────────────────────────────────────────────────────────────────────────
EXCLUSION RULES — FALSE_POSITIVE if any match (cite the rule number):

 1. Volumetric DoS or missing rate-limiting (infrastructure concern).
    ReDoS, algorithmic complexity, and unbounded recursion ARE valid.
 2. Test-only, dead, example, or fixture code; crash with no security impact.
 3. Intended design (compression middleware, weak algorithm alongside a
    strong one for backward compatibility).
 4. Memory-safety concern in a memory-safe language outside unsafe/FFI.
 5. SSRF where attacker controls only the path, not host or protocol.
 6. User input flowing into an AI/LLM prompt (not a code vulnerability).
 7. Path traversal in object storage (S3/GCS) where `../` cannot escape a
    trust boundary.
 8. Trusted operator inputs as attack vector (env vars, CLI flags), UNLESS
    the ENVIRONMENT marks them untrusted.
 9. Client-side code flagged for server-side vulnerability classes.
10. Outdated dependency versions (managed separately).
11. Weak random used for non-security purposes (jitter, shuffling, dev fallbacks).
12. Low-impact nuisance: log spoofing, CSRF on logout, self-XSS, tabnabbing,
    open redirect, regex injection.
13. Missing hardening or best-practice gap with no concrete exploit path.
14. XSS in a framework with default auto-escaping (React, Angular, Vue,
    Jinja2 autoescape=on) unless the sink is a raw-HTML escape hatch.
15. Unguessable identifiers (UUIDv4, 128-bit+ random tokens) flagged as
    predictable.
16. Race conditions or TOCTOU that are theoretical only with no realistic
    window and no security-relevant state change.

{if context.extra_fp_rules: append under "ORG-SPECIFIC RULES:"}

────────────────────────────────────────────────────────────────────────
VERDICT — end your response with EXACTLY this block:

  VERDICT: TRUE_POSITIVE | FALSE_POSITIVE | CANNOT_VERIFY
  CONFIDENCE: <0-10>
  REFUTE_REASON: <doesnt_exist | already_handled | implausible_trigger |
    intentional_behavior | misread_code | duplicate | not_actionable | n/a>
  EXCLUSION_RULE: <1-16, org rule, or none>
  FIRST_LINK: <file:line of the first call site you read, or "none found">
  RATIONALE: <2-5 sentences citing specific file:line evidence>

TRUE_POSITIVE requires ALL of: reachable from untrusted input; protections
insufficient or bypassable; real-world exploitation feasible.

FALSE_POSITIVE requires ANY of: unreachable from untrusted input; adequately
protected on all paths; scanner misread the code; exclusion rule applies.

CANNOT_VERIFY: static reasoning genuinely hit its limit (runtime config,
binary not inspectable). Use sparingly.
```

### 3b. Spawn N verifiers per candidate

For each finding in `candidates[]`, build N Task calls (`subagent_type:
"general-purpose"`, `description: "verify {id} vote {k}/{N}"`), all in one
message so they run concurrently.

**Always set `subagent_type`.** A fork inherits the full conversation context
and defeats verifier independence.

Each prompt is the 3a template with this block appended:

```
────────────────────────────────────────────────────────────────────────
FINDING UNDER REVIEW (treat as a CLAIM, not a fact):

  id:        {id}
  file:      {file}
  line:      {line}
  category:  {category}
  severity (claimed): {severity}
  title:     {title}

  description:
  {description}

  exploit_scenario:
  {exploit_scenario or "(not provided)"}

  preconditions (claimed):
  {preconditions as bullets or "(not provided)"}

You are vote {k} of {N}. You have NOT seen the other verifiers' reasoning.
Work independently from the code.
```

If `len(candidates) * N` exceeds ~40, shard into sequential batches of ~40,
but keep each batch a single message.

**Compact form** — use when `candidates × votes > ~50` to control prompt size:

```
Adversarially verify ONE scanner finding. Default: scanner is WRONG.
Read-only, scoped to {REPO_PATH} only. No exec, no network.
ENVIRONMENT: {context.environment}

Steps: (1) Read {file}:{line} yourself; do not trust the description.
(2) Trace callers backwards; quote the first call-site file:line.
(3) Hunt for protections: validation, escaping, type bounds, auth gates,
dead/test code. (4) Stress-test each protection on every path.

Exclusion rules (FALSE_POSITIVE if matched): 1 volumetric DoS; 2 test/dead/
fixture; 3 intended design; 4 memory-safety in safe lang outside unsafe/FFI;
5 SSRF path-only; 6 LLM input; 7 object-storage traversal; 8 trusted
operator env/CLI; 9 client code server vuln; 10 outdated deps; 11 weak
random non-security; 12 low-impact nuisance; 13 missing-hardening-only;
14 XSS in auto-escape framework without raw-HTML hatch; 15 unguessable
UUID/token; 16 theoretical-only race/TOCTOU.
{+ org rules if any}

End with EXACTLY:
  VERDICT: TRUE_POSITIVE | FALSE_POSITIVE | CANNOT_VERIFY
  CONFIDENCE: <0-10>
  REFUTE_REASON: <doesnt_exist|already_handled|implausible_trigger|
    intentional_behavior|misread_code|duplicate|not_actionable|n/a>
  EXCLUSION_RULE: <1-16, org rule, or none>
  FIRST_LINK: <file:line or "none found">
  RATIONALE: <2-5 sentences, file:line cited>

FINDING: {id} {file}:{line} {category} (claimed {severity})
{title}
{description}
Vote {k}/{N}. Independent — do not seek other votes.
```

Findings with a `file` but no `line` receive one verifier vote regardless of
`--votes`.

If a Task call returns `status: "async_launched"`, parse VERDICT blocks from
completion notifications as they land, or re-spawn the missing verifiers in a
smaller shard if notifications do not arrive.

### 3c. Tally votes

For each candidate, parse the trailing VERDICT block from each verifier
(tolerate code fences and whitespace). If a verifier errored or produced no
parseable block, re-spawn it once. If the retry also fails, count that vote
as `cannot_verify` with `confidence: 0` and note `"verifier_error"`.

Build:
- `vote_breakdown`: `{"true_positive": x, "false_positive": y, "cannot_verify": z}`
- `confidence`: mean CONFIDENCE across votes agreeing with the majority,
  rounded to one decimal.
- `exclusion_rule`: modal EXCLUSION_RULE among FALSE_POSITIVE votes, else null.
- `refute_reasons`: sorted unique REFUTE_REASON values from FALSE_POSITIVE votes.
- `first_links`: unique FIRST_LINK values across all votes.
- `rationale`: RATIONALE from the highest-confidence vote on the winning side.

**Decide verdict:**
- Majority TRUE_POSITIVE → `verdict: true_positive`. Proceeds to Phase 4.
- Majority FALSE_POSITIVE → `verdict: false_positive`. Skips Phase 4.
- No majority (tie or majority CANNOT_VERIFY):
  - Precision → `verdict: false_positive`; append `"(split vote, dropped
    under precision policy)"` to rationale.
  - Recall → `verdict: true_positive` with `verify_verdict: needs_manual_test`.
  - Ask → collect all split findings, present in one AskUserQuestion call
    (keep / drop per finding), then apply choices.

`confirmed[]` = candidates with `verdict == true_positive`.

**Per-candidate sharding** (when `len(candidates) * votes > ~40`): after
tallying each candidate's votes, additionally write:
1. Write tool → `./.triage-state/_chunk.tmp` with that finding's post-tally dict.
2. `python3 .claude/skills/_lib/checkpoint.py shard ./.triage-state <id> --from ./.triage-state/_chunk.tmp`

On resume at `phase_done == 2`, read `progress.json:shards_done` and spawn
verifiers only for candidates NOT yet in `shards_done`. Once all candidates
are sharded, write the consolidated `phase3.json` checkpoint.

**Checkpoint payload:**
```json
{"phase": 3, "context": {...}, "findings": [{all findings with verdict/vote_breakdown/confidence/refute_reasons/first_links/rationale/exclusion_rule}], "confirmed": ["f001", ...]}
```

---

## Phase 4: Rank by exploitability

Recompute severity from preconditions and reachability, independent of the
scanner's claim. "This is real" must not automatically inflate to "this is
critical."

### 4a. Ranking prompt

Spawn one Task per confirmed finding (all in one message):

```
You are assigning severity to a CONFIRMED security finding. Verification
already happened; assume it is real. Derive how bad it is, independently of
what the scanner claimed.

You may Read/Grep {REPO_PATH} to check preconditions. Do NOT execute code.

ENVIRONMENT: {context.environment}
THREAT MODEL: {context.threat_model as bullets, or "(none)"}
SCORING STANDARD: {context.scoring}

FINDING:
  id:        {id}
  file:      {file}:{line}
  category:  {category}
  claimed severity: {severity}
  reachability evidence: {first_links from Phase 3}
  verifier rationale: {rationale from Phase 3}

────────────────────────────────────────────────────────────────────────
STEP 1: List EVERY precondition that must hold for exploitation. Be concrete:
required auth state, configuration, prior request, race window, attacker
position. State the minimum ACCESS LEVEL (unauthenticated remote /
authenticated / local / physical).

STEP 2: Derive severity:

  | Preconditions | Access required           | Severity |
  |---|---|---|
  | 0             | Unauthenticated remote    | HIGH     |
  | 1–2           | Authenticated             | MEDIUM   |
  | 3+            | Local-only / no demo path | LOW      |

  Evaluate each column independently; take the LOWER result. If your
  preconditions list has 3+ items, HIGH is almost certainly wrong.

STEP 3: Threat-model match. If the THREAT MODEL is non-empty and this finding
maps to an entry, note which one. A match may raise severity by ONE step
(LOW→MEDIUM or MEDIUM→HIGH), never two.

STEP 4: Judge the scanner's claimed severity on a -5..+5 scale:
  +3..+5  claimed severity is justified or understated
   0..+2  roughly right
  -1..-3  inflated by one level
  -4..-5  badly inflated (LOW dressed as HIGH)

STEP 5: verify_verdict — exactly one of:
  exploitable        preconditions are realistically satisfiable
  mitigated          a deployed control reduces it below the derived severity
                     (name the control)
  needs_manual_test  severity depends on something only a runtime test can settle

STEP 6: If SCORING STANDARD is CVSS or OWASP, emit a `severity_label` in that
format. Otherwise set it equal to the derived HIGH/MEDIUM/LOW.

────────────────────────────────────────────────────────────────────────
Respond with ONLY this block:

  PRECONDITIONS:
  - <one per line>
  ACCESS_LEVEL: <unauthenticated_remote|authenticated|local|physical>
  SEVERITY: <HIGH|MEDIUM|LOW>
  SEVERITY_LABEL: <per scoring standard>
  THREAT_MATCH: <matched entry, or none>
  SEVERITY_ALIGNMENT: <-5..+5>
  VERIFY_VERDICT: <exploitable|mitigated|needs_manual_test>
  RANK_RATIONALE: <2-4 sentences>
```

### 4b. Merge

Parse each response and attach `preconditions`, `access_level`, `severity`
(recomputed), `severity_label`, `threat_match`, `severity_alignment`,
`verify_verdict`, and append RANK_RATIONALE to `rationale`.

For findings not reaching Phase 4 (false_positive, duplicate, unlocatable):
set `severity: null`, `verify_verdict: null`, `severity_alignment: null`,
`preconditions: []`.

**Checkpoint payload:**
```json
{"phase": 4, "context": {...}, "findings": [{all findings with ranking fields}]}
```

---

## Phase 5: Route

Tag each confirmed true-positive with the most specific owner inferable.
For each finding in `confirmed[]`, stop at the first hit:

1. **CODEOWNERS / OWNERS**: grep `--repo` for `CODEOWNERS`, `OWNERS`,
   `.github/CODEOWNERS`, `docs/CODEOWNERS`. If found, match the finding's
   `file` against its patterns (last match wins). Hint:
   `"CODEOWNERS: <pattern> → <owner(s)>"`.
2. **git log**: if the repo has git history, run
   `git -C {REPO} log --format='%an' -n 50 -- "{file}" | sort | uniq -c | sort -rn | head -3`.
   Hint: `"top committer: <name> (<n>/<total> recent commits); no CODEOWNERS entry"`.
3. **Module fallback**: `"component: <top-level dir of file>/; no CODEOWNERS or git history"`.

Attach as `owner_hint`. Make the source explicit. For non-true-positive
findings, set `owner_hint: null`.

**Checkpoint payload:**
```json
{"phase": 5, "context": {...}, "findings": [{all findings with owner_hint}]}
```

---

## Phase 6: Output

### 6a. Sort

Order all findings by:
1. `verdict`: true_positive, then duplicate, then false_positive.
2. Within true_positives: severity HIGH > MEDIUM > LOW, then `confidence`
   descending, then `severity_alignment` descending.
3. Within others: original `id`.

### 6b. Write `./TRIAGE.json`

```json
{
  "triage_completed": true,
  "triage_context": {
    "mode": "interactive|auto",
    "environment": "...",
    "threat_model": ["..."],
    "scoring": "...",
    "noise_tolerance": "...",
    "votes_per_finding": 3,
    "repo": "..."
  },
  "summary": {
    "input_count": 0,
    "duplicates": 0,
    "false_positives": 0,
    "true_positives": 0,
    "needs_manual_test": 0,
    "by_severity": {"HIGH": 0, "MEDIUM": 0, "LOW": 0}
  },
  "findings": [
    {
      "id": "f001",
      "source": "VULN-FINDINGS.json#0",
      "title": "...",
      "file": "...",
      "line": 0,
      "category": "...",
      "claimed_severity": "HIGH",
      "verdict": "true_positive|false_positive|duplicate",
      "verify_verdict": "exploitable|mitigated|needs_manual_test|null",
      "confidence": 0.0,
      "severity": "HIGH|MEDIUM|LOW|null",
      "severity_label": "...",
      "severity_alignment": 0,
      "preconditions": ["..."],
      "access_level": "...",
      "threat_match": "...|null",
      "rationale": "...",
      "vote_breakdown": {"true_positive": 0, "false_positive": 0, "cannot_verify": 0},
      "refute_reasons": ["..."],
      "exclusion_rule": null,
      "first_links": ["file:line"],
      "duplicate_of": null,
      "absorbed": ["..."],
      "owner_hint": "...",
      "missing_fields": ["..."]
    }
  ]
}
```

Every input finding appears exactly once. Do not silently drop anything. Do
not print this JSON to the terminal; write to file only.

### 6c. Write `./TRIAGE.md`

Build incrementally — one Write per section, never the whole file at once. A
stalled write loses only that section, not the file.

**Step 1 — header**: Write tool → `./TRIAGE.md` (clobbers prior):
```
# Triage Report

{N} in → {D} duplicates, {F} false positives, {T} confirmed ({H} high / {M} med / {L} low), {X} need manual test

Context: {mode}; environment = {environment}; scoring = {scoring}; {votes}-vote verification.

## Act on these
```

**Step 2 — per finding**: for each true_positive in severity order:
1. Write tool → `./.triage-state/_chunk.tmp`:

```
### [{severity}] {title}  ({id})
`{file}:{line}` | {category} | claimed {claimed_severity} (alignment {severity_alignment:+d}) | confidence {confidence}/10
**Owner:** {owner_hint}
**Verdict:** {verify_verdict}, votes {vote_breakdown}
**Preconditions ({n}):** {bulleted list}
**Threat-model match:** {threat_match or "none"}
**Why:** {rationale}
**Reachability evidence:** {first_links}
{if verify_verdict == needs_manual_test:}
> Recommend a human build a PoC; static reasoning reached its limit.
```

2. `python3 .claude/skills/_lib/checkpoint.py append ./TRIAGE.md --from ./.triage-state/_chunk.tmp`

**Step 3 — footer**: Write tool → `./.triage-state/_chunk.tmp`:
```
## Dropped

| id | title | file:line | why dropped |
{false_positives: refute_reasons + exclusion_rule}
{duplicates: "duplicate of {duplicate_of}"}
{unlocatable: "no source location in input"}
```
Then `checkpoint.py append ./TRIAGE.md --from ./.triage-state/_chunk.tmp`.

**Final checkpoint:**
```
python3 .claude/skills/_lib/checkpoint.py done ./.triage-state 6
```

### 6d. Terminal summary

```
Triage complete: {N} findings → {T} confirmed, {F} false positives, {D} duplicates.

  HIGH:   {n}   {title of top HIGH, owner_hint}
  MEDIUM: {n}
  LOW:    {n}
  Needs manual test: {n}

  Top refute reasons: {top 3 with counts}

Wrote ./TRIAGE.md and ./TRIAGE.json
```
