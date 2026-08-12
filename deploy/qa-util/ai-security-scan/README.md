# AI Security Scan

AI-assisted security review of the NL Wallet repository, built on Claude Code
skills. Only **read-only** skills are included: threat modelling, vulnerability
scanning (full tree and diff), and triage.

## Contents

| Skill | What it does | Output |
|---|---|---|
| `/threat-model` | Builds a threat model from the code, git history and advisories (`bootstrap`), from an owner interview (`interview`), or both | `THREAT_MODEL.md` |
| `/vuln-scan` | Static vulnerability review of a directory; uses `THREAT_MODEL.md` for focus areas when present | `VULN-FINDINGS.json` + `.md` |
| `/vuln-scan-diff` | Same review, scoped to a git commit range (for MRs / release diffs) | `VULN-FINDINGS-DIFF.json` + `.md` |
| `/triage` | Verifies, deduplicates, re-ranks and routes a pile of findings (from the scans above or any other scanner) | `TRIAGE.json` + `.md` |

Supporting pieces:

- `.claude/skills/_lib/checkpoint.py` — shared checkpoint helper; lets
  `/threat-model` and `/triage` resume after an interruption instead of
  starting over.
- `.claude/settings.json` — the tool allowlist governing every run. It
  permits only read-only operations (Read/Glob/Grep/Write/Task plus a narrow
  set of Bash commands the skills need) and uses `defaultMode: default`, so
  anything not on the list — `kubectl`, `curl`, `env`, arbitrary `python3` —
  is denied. This is the enforcement point for the isolation described below.
- `false-positives.md` — reviewed-and-accepted findings that `/triage`
  should suppress instead of re-raising (see below).
- `docs/` — background on the security model and the triage methodology.

All skills only *read* the code under review. They never build it, run it,
install dependencies, or touch the network beyond the Claude API, so no
sandbox is required. See [docs/security.md](docs/security.md).

## Running locally

Start Claude Code in this directory so the skills are project-scoped, then
point them at the repository root (`../../..`):

```bash
cd deploy/qa-util/ai-security-scan
claude --sandbox

# Full scan of a workspace member, seeded by the threat model if present
> /vuln-scan ../../../wallet_core/lib/jwt
# → writes ../../../wallet_core/lib/jwt/VULN-FINDINGS.json + .md

# Scan only what a branch changed
> /vuln-scan-diff ../../.. main..HEAD
# → writes ../../../VULN-FINDINGS-DIFF.json + .md

# Verify, dedupe and rank the findings
> /triage ../../../wallet_core/lib/jwt/VULN-FINDINGS.json --repo ../../../wallet_core/lib/jwt
```

`/threat-model bootstrap <dir>` should be run locally **before every
release** and the resulting `THREAT_MODEL.md` committed alongside the code
under review. Both scan skills (`/vuln-scan`, `/vuln-scan-diff`) pick it up
automatically from the target directory and scan more precisely when it is
present. Example for the Rust core:

```bash
cd deploy/qa-util/ai-security-scan
claude --sandbox
> /threat-model bootstrap ../../../wallet_core
```

Then commit `wallet_core/THREAT_MODEL.md` as part of the release preparation.

Scan outputs (`VULN-FINDINGS*`, `TRIAGE*`) are written into the target
directory (`/vuln-scan`, `/vuln-scan-diff`) or alongside the input findings
file (`/triage`), and are gitignored — they are review input, not repository
content.

## Running in CI (GitLab)

The scans run as their **own pipeline**, defined in
`deploy/gitlab/ai-security-scan.yml`. They are deliberately kept out of the
push, merge request, and normal `main` pipelines: the jobs only materialize
when a pipeline is started **from the GitLab web UI** with `SCHEDULED` set to
the specific scan type.

To run a scan, start a pipeline — via **Build → Pipelines → Run pipeline** —
select the branch, and add:

| Variable | Value |
|---|---|
| `SCHEDULED` | `ai-security-scan-diff` to run the diff scan, or `ai-security-scan-full` to run the full scan (required — selects which scan job and the token fetch job are included) |
| `TRIAGE` | `true` to also verify, dedupe and rank the findings. Defaults to `true` for `ai-security-scan-diff` and `false` for `ai-security-scan-full`. |
| `SCAN_MODEL` | Claude model ID to use for all scan and triage invocations (default `claude-sonnet-4-6`). Use `claude-haiku-4-5-20251001` for a faster, cheaper run or `claude-opus-4-8` for maximum thoroughness. Avoid `claude-fable-5` for security scanning — its safety classifiers target cybersecurity content and may refuse scan requests. |
| `CLAUDE_CODE_MAX_TURNS` | Maximum number of agentic turns per `claude` invocation (default `50`). Acts as a safety ceiling — a normal scan completes well within this. Raise it if a large full scan is cut short; lower it to constrain cost on narrow diff scans. |
| plus any per-job variable below | |

Both jobs run the scan headless via `claude -p` and collect the reports as
job artifacts (30 days). With `TRIAGE=true`, `TRIAGE.md` is the
human-readable end product, sorted by what actually needs attention.

| Job | What it scans | Variables |
|---|---|---|
| `ai-security-scan-diff` | Code changed in a commit range | `SCAN_COMMIT_RANGE` — any range `git diff` accepts (default: the last commit, `HEAD~1..HEAD`) |
| `ai-security-scan-full` | A source tree (defaults to the Rust core) | `SCAN_TARGET` — path relative to the repo root (default `wallet_core`; a narrower target like `wallet_core/lib/jwt` is faster and cheaper), `SCAN_EXCLUDES` — `--exclude` flags, relative to `SCAN_TARGET` (default drops `target`, `tests_integration`, `demo`, `gba_hc_converter`, `wallet_server/pid_issuer`) |

> **Keep the diff range off merge commits.** `git diff A..B` compares the two
> tree snapshots directly, so a range that spans a merge (e.g. `HEAD~5..HEAD`
> when one of those commits merged `main`) pulls in the entire merged-in
> history — hundreds of files — and turns a diff scan into a full-repo scan.
> For `ai-security-scan-diff`, use a linear range of the actually-new commits
> (e.g. `<last-release-tag>..HEAD` or the tip commits above the last merge). To
> deliberately review a large merged chunk, use `ai-security-scan-full` with a
> scoped `SCAN_TARGET` instead.

The Claude Code CLI is installed in the job with
`npm i -g @anthropic-ai/claude-code` on the standard `ci-node` image.

### Isolation

The scan jobs run untrusted repository content through the agent (the
prompt-injection surface), so they are deliberately kept away from cluster
secrets:

- **Only `ai-security-scan-fetch-token` touches the cluster.** It extends
  `.env-k8s`, reads the Anthropic API key from the `nl-wallet-anthropic`
  secret (key `claude_code_oauth_token`), and passes it to the scan jobs as a
  `dotenv` variable (`CLAUDE_CODE_OAUTH_TOKEN`). The scan jobs do **not** extend
  `.env-k8s`, so they have no kubeconfig and cannot enumerate namespace
  secrets.
- **The agent's tools are restricted** by `.claude/settings.json`
  (`defaultMode: default`, no `bypassPermissions`): only read-only operations
  are allowed, so even with the key present the agent cannot invoke
  `kubectl`, `curl`, `env`, or arbitrary `python3` to move a secret.
- **The key authenticates to Claude only**, not to any wallet
  infrastructure. It is org-billed and usage-metered, so scope it minimally
  in the Anthropic Console (single workspace) and set a spend cap — a leaked
  key can run up billing but cannot reach wallet systems. Rotate it
  periodically. The `dotenv` value is not auto-masked, so the scan steps must
  never echo it.

### When to run

Trigger them at the cadence below via **Build → Pipelines → Run pipeline**
and set the variables per the table.

| Cadence | Where | `SCHEDULED` / action | Additional variables | Purpose |
|---|---|---|---|---|
| **Every release** | local | run `/threat-model bootstrap` | — | Refresh `THREAT_MODEL.md` and commit it before the full scan so the scanner has up-to-date focus areas. |
| **Every sprint** | CI | `ai-security-scan-diff` | `SCAN_COMMIT_RANGE` covering the sprint's commits (triage runs by default) | Review what changed during the sprint and get a verified, ranked list to act on. |
| **Every release** | CI | `ai-security-scan-full` | `TRIAGE=false` (default) | A full-tree sweep before shipping; leave the raw findings for a reviewer to go through (run `/triage` manually on the artifact if a pass is warranted). |

For the sprint run, set `SCAN_COMMIT_RANGE` to the range of the sprint — for
example the previous release tag (or the commit at the start of the sprint)
up to `HEAD`, such as `v1.2.0..HEAD`.

## Reading the results

- Treat `VULN-FINDINGS*` as raw scanner output: it intentionally biases
  toward recall and contains false positives. Run `/triage` before spending
  engineering time on it.
- In `TRIAGE.md`, the "Act on these" section is the ranked, verified list;
  the "Dropped" table explains every rejection so a human can audit them.
- A `needs_manual_test` verdict means static reasoning hit its limit: a
  human should build a controlled proof of concept.
- All output is a starting point for human review, never a substitute for
  it.

## Suppressing accepted findings

Some findings are real observations but not actionable — genuine false
positives, or accepted-risk design decisions (e.g. a service that
deliberately uses plain HTTP on an internal network). Record these once in
[`false-positives.md`](false-positives.md) so `/triage` marks them
`false_positive` instead of re-raising them every run.

Both CI jobs already pass this file to triage
(`/triage … --fp-rules false-positives.md`), and you can do the same
locally:

```bash
> /triage VULN-FINDINGS.json --repo ../../.. --fp-rules false-positives.md
```

To accept a new finding, add a rule to `false-positives.md` following the
format documented at the top of that file: scope it to the specific
file/component and class of finding, and say why it is accepted (with a
ticket reference where one exists). Suppression happens at the triage step —
`/vuln-scan` still reports these findings; triage is what drops them. Review
additions the way you would any security exception: a too-broad rule can
hide a genuine future bug.

## Documentation

- [docs/security.md](docs/security.md) — why the skills are safe to run
  unsandboxed, and the caveats that remain (prompt injection, credentials).
- [docs/triage.md](docs/triage.md) — how triage verifies, dedupes and ranks.
- Per-skill documentation: `.claude/skills/<skill>/README.md` and the
  `SKILL.md` files themselves (arguments, behavior, output schemas).
