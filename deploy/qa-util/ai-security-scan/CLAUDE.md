# AI Security Scan

Claude Code skills for AI-assisted security review of the NL Wallet
repository. All skills are **read-only with respect to the target**: they
read and write files in the repo (reports, state) but never build, run, or
network-probe the code under review.

Available skills (`.claude/skills/`):

- `/threat-model` — bootstrap, interview, or bootstrap-then-interview →
  `THREAT_MODEL.md`
- `/vuln-scan` — static review of a directory → `VULN-FINDINGS.json` + `.md`
- `/vuln-scan-diff` — same, scoped to a git commit range →
  `VULN-FINDINGS-DIFF.json` + `.md`
- `/triage` — verify + dedupe + rank a findings pile → `TRIAGE.json` + `.md`

The NL Wallet repository root is `../../..` relative to this directory. See
`README.md` for usage and the CI setup.
