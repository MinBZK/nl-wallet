---
name: threat-model
description: >-
  Build a threat model for a target codebase. Three modes: "bootstrap" derives
  a threat model from code and past vulnerability history; "interview" walks an
  application owner through a structured four-question session; "bootstrap-then-interview"
  chains the two when both the codebase and its owner are available. All modes
  write THREAT_MODEL.md to the target directory in a shared schema. Use when asked
  to "threat model", "build a threat model", "map the attack surface", or
  "what should we be worried about in this codebase".
argument-hint: "[bootstrap-then-interview|bootstrap|interview] <target-dir> [--vulns <file>] [--design-doc <file>] [--seed <THREAT_MODEL.md>] [--fresh]"
allowed-tools:
  - Read
  - Glob
  - Grep
  - Write
  - Bash(python3 .claude/skills/_lib/checkpoint.py:*)
  - Bash(git log:*)
  - Bash(git diff:*)
  - Bash(git show:*)
  - Bash(git rev-parse:*)
  - Bash(git remote:*)
  - Bash(git -C:*)
  - Bash(find:*)
  - Bash(ls:*)
  - Bash(cat:*)
  - AskUserQuestion
  - Task
---

# /threat-model

A threat model answers the question: **"what could go wrong with this system,
who would cause it, and what should we do about that?"** It is built before
specific bugs are known, not as a catalogue of them. A good threat model gives
`/vuln-scan` its focus areas and tells `/triage` which findings matter most.

**The patch test:** if fixing a single line of code makes an entry disappear, it
was a vulnerability, not a threat. A threat — "attacker achieves RCE via
untrusted document parsing" — remains valid even after every known bug in that
surface is patched. Vulnerabilities appear in this skill only as **evidence**
that a threat class is fertile and raises its likelihood score.

**Usage:** `/threat-model [bootstrap-then-interview|bootstrap|interview] <target-dir> [flags]`

---

## Step 0 — Safety declaration (runs before anything else)

This skill does **static analysis only**. It reads source code, git history,
and any advisory or vulnerability files the user supplies, then writes
`<target-dir>/THREAT_MODEL.md`. It does not build, execute, or fuzz the target,
and does not make network requests to the target's infrastructure.

Before doing anything else, confirm and state in your first response:

1. The target directory exists and is a local checkout you can read.
2. You will not execute any code found inside the target directory.
3. If you are asked to "fetch CVEs" or `--vulns` contains a URL, you will only
   read advisory data that is already on disk — do not reach external services.

If the user asks you to validate a threat by running a proof-of-concept, decline.
Validation belongs in a controlled manual test.

---

## Step 1 — Select a mode

Parse `$ARGUMENTS`. The first token selects the mode:

| First token | Action |
|---|---|
| `bootstrap` | Read `bootstrap.md` in this directory and follow it. |
| `interview` | Read `interview.md` in this directory and follow it. |
| `bootstrap-then-interview` | Run bootstrap to completion, then immediately continue into interview mode seeded from the resulting draft. |
| anything else, or empty | Ask the user: **"Is someone who built or owns this system available to answer questions in this session?"** Then recommend: yes + codebase present → `bootstrap-then-interview`; yes + no codebase → `interview`; no → `bootstrap`. |

All three modes write the same artifact (`THREAT_MODEL.md`, format specified in
`schema.md`) so downstream skills do not need to know which mode produced it.

| | `bootstrap` | `interview` |
|---|---|---|
| **Requires** | A local checkout; optionally past vulns | An owner present in the session |
| **Method** | Parallel research swarm → synthesize sections 1-3 → cluster vulns into threat classes → STRIDE gap-fill → emit | Four-question owner session grounded in code where possible |
| **Best for** | Inherited or third-party code, systems with a CVE history | New designs, systems where risk lives in business logic |

**Context durability.** Interview mode spans many turns; tool results from early
reads may be evicted before you need them.

- Read `interview.md` or `bootstrap.md` **at the point you need each section**,
  not up front all at once.
- If a re-read with the Read tool reports "file unchanged" (cached result
  evicted), reload via `cat <path>` in Bash.

**Interview backbone** (available if `interview.md` cannot be reloaded):

| Question | Fills |
|---|---|
| Q1: What are we working on? | Section 1 context, section 2 assets, section 3 entry points |
| Q2: What can go wrong? | Section 4 threats (id, threat, actor, surface, asset) |
| Q3: What are we going to do about it? | Section 4 impact / likelihood / status / controls; section 5 deprioritized; section 8 mitigations |
| Q4: Did we do a good job? | Ranking validation, coverage check, section 6 open questions |

### `bootstrap-then-interview`

When an owner is available and the codebase is checked out, this is the
recommended path: the owner's time goes to refining a code-grounded draft
rather than describing the system from scratch.

1. Tell the owner: "I'll read the code first and draft a threat model (about
   5-10 minutes), then we'll walk it together to correct and extend it. Want
   that, or would you rather start cold?" Proceed only if they agree; otherwise
   fall back to interview mode.
2. Read `bootstrap.md` and follow it end-to-end. Write
   `<target-dir>/THREAT_MODEL.md`.
3. Immediately continue into interview mode: read `interview.md` and follow it
   with `--seed <target-dir>/THREAT_MODEL.md`. The section 6 open questions from
   the bootstrap become the starting prompts for Q1–Q4; the owner confirms,
   corrects, and extends rather than building from nothing.
4. Overwrite `<target-dir>/THREAT_MODEL.md` with the refined model.

---

## Step 2 — Shared output contract

Every mode writes `<target-dir>/THREAT_MODEL.md` conforming to `schema.md`.
**Read `schema.md` immediately before writing the file**, not at routing time —
in interview mode the gap between routing and emit can be many turns, and an
early read will be evicted.

After writing, print to the user:

1. The path to `THREAT_MODEL.md`.
2. The top 5 threats ranked by likelihood × impact (id, one-line description).
3. For `bootstrap`: any open questions the code could not answer (these seed a
   later `interview` pass).
4. For `interview`: any owner statements that could not be verified in code
   (these seed follow-up code review).
