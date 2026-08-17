# Security considerations

> **TL;DR:** Every skill in this component (`/threat-model`, `/vuln-scan`,
> `/vuln-scan-diff`, `/triage`) only reads the target code and writes report
> files. Nothing builds, runs, or tests the target, so no sandbox is
> required. The residual risks are prompt injection from the code being
> reviewed and credential exposure on the machine running the scan.

## What the skills can and cannot do

Each skill's `SKILL.md` declares an explicit tool allowlist: `Read`, `Glob`,
`Grep`, `Write`, subagents, and a narrow set of Bash commands (`git`, `rg`,
`jq`, `find`, and `python3 .claude/skills/_lib/checkpoint.py` for state I/O).
`.claude/settings.json` enforces the same allowlist for every run with
`defaultMode: default` (not `bypassPermissions`), so anything off the list —
`kubectl`, `curl`, `env`, arbitrary `python3` — is denied. The safety
property that matters is **no execution of target code**: the wallet's own
sources are never compiled, run, or imported during a scan.

The upstream project also shipped an autonomous pipeline that *does* execute
target code (to reproduce crashes under ASAN) and therefore required a gVisor
sandbox. That pipeline has been removed from this copy; if you ever reintroduce
execution-based verification, bring back the sandboxing with it.

## Residual risks

**Prompt injection from the reviewed code.** The scan reads repository
content, and repository content can contain adversarial instructions
(in comments, strings, docs, or dependency code). A hijacked scan could
mis-report findings or write misleading reports — it cannot do more than
that, because the tool allowlist has no arbitrary command execution and no
network access beyond the Claude API. Consequences:

- Findings are *claims*, not facts, until `/triage` (which adversarially
  re-verifies against source) and ultimately a human confirms them.
- Be more suspicious when scanning third-party/vendored code than when
  scanning our own tree.

**Credentials on the scan machine.** The scan process can read whatever the
user running it can read. On CI, run it in an ephemeral job with only the
`CLAUDE_CODE_OAUTH_TOKEN` it needs; don't mount other secrets into the job.
Locally, the standing advice for any agent applies: don't run it in a shell
session holding credentials it doesn't need.

**Output handling.** `VULN-FINDINGS*`/`TRIAGE*` files describe potential
vulnerabilities in this codebase. Until fixed, treat them like any other
vulnerability report: keep them in CI artifacts with appropriate access,
not in public places.

## Permissions configuration

`.claude/settings.json` pre-approves the read-only commands the skills use
(`allow` list) and runs with `defaultMode: default` — it does **not** use
`bypassPermissions`. A tool that is neither on the allowlist nor approved is
denied, so the agent cannot run `kubectl`, `curl`, `env`, or arbitrary
`python3` even in an unattended CI run. Keep the allowlist read-only: adding
a command that reaches the network or executes target code would undermine
the isolation the scan jobs rely on (see the README's Isolation section).
