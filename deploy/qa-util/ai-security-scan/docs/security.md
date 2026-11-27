# Security considerations

> **TL;DR:** Every skill in this component (`/threat-model`, `/vuln-scan`,
> `/vuln-scan-diff`, `/triage`) only reads the target code and writes report
> files. Nothing builds, runs, or tests the target, so no sandbox is
> required. The residual risks are prompt injection from the code being
> reviewed and credential exposure on the machine running the scan.

## What the skills can and cannot do

`.claude/settings.json` declares the tool rules for every run. The skills do
not declare `allowed-tools`: Claude Code adds those rules to the settings,
also in `dontAsk` mode, so a broad rule in a skill opens the allowlist. The
rules permit writes to the output files and state directories only,
`git diff`, `git log`, `git show`, `git rev-parse`, some file inspection
commands (`grep`, `ls`, `wc`, `head`, `file`, `cat`), and
`python3 .claude/skills/_lib/checkpoint.py` for state I/O. The skills
never compile, run or import the wallet's own sources.

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

**Credentials on the scan machine.** The scan process can read the files
that the user who runs it can read. In CI, the job receives all unprotected
project and group CI variables. `run-skill.sh` starts the agent with only the
variables that Claude Code needs. Locally, the agent gets the environment of
your shell. Do not run it in a shell that holds credentials it does not need.

**Output handling.** `VULN-FINDINGS*`/`TRIAGE*` files describe potential
vulnerabilities in this codebase. Until fixed, treat them like any other
vulnerability report: keep them in CI artifacts with appropriate access,
not in public places.

## Permissions configuration

In CI, `run-skill.sh` starts Claude Code with `--permission-mode dontAsk`.
Claude Code then denies each tool call that the rules do not allow. Locally,
`defaultMode: default` applies: Claude Code asks you before it runs other
tool calls, for example the write of `THREAT_MODEL.md` into the target
directory.

The `allow` list is a list of prefixes. A `Bash(<prefix>:*)` rule allows
each command that starts with the prefix, with all arguments after it. Many
commands have options that run other commands or write files.

With the current rules, Claude Code denies commands like the following:

- `Bash(git -C:*)`. This rule matches each git command that starts with
  `-C <dir>`, with all global options and subcommands after it. For example,
  `git -C . -c alias.x='!cmd' x` sets an alias with a `-c` config override,
  and git runs `cmd` in a shell. Rules for a subcommand, such as
  `Bash(git log:*)`, do not match a command with options before the
  subcommand, so the skills run `git log` without `-C`.
- `Bash(rg:*)`, because `rg --pre <cmd>` runs a command for each file.
- `Bash(find:*)`, because `-exec`, `-execdir` and `-ok` run commands, and
  `-delete` and `-fprint` change files.
- `Bash(jq:*)`, because `jq -n '$ENV'` prints the environment.
- `Write` without a path, because the agent can then write files outside the
  repository. A changed `~/.gitconfig`, for example, can make the next git
  command run a command.
- `Read` without a path, because the agent can then read files outside the
  repository, for example `~/.ssh`. Reads in the working directories,
  subagents and questions to the user need no rule.

The allowed `git diff`, `git log` and `git show` accept `--output=<file>`,
which writes a file. The `deny` rule `Bash(git * --output*)` blocks it.

Claude Code itself blocks `find -exec`, reads of `/proc/*/environ`, and
writes into `.git/` and `.claude/`.

When you change the rules or `CLAUDE_CODE_VERSION`, test the rules again.
Make a scratch git repository with a copy of `.claude/settings.json`, and ask
`claude -p --permission-mode dontAsk --output-format json` to run commands
like the ones above. The `permission_denials` field of the output shows which
tool calls Claude Code denied. Also make sure that the command did not run,
for example with a marker file.
