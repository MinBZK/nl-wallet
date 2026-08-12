# threat-model

A Claude Code skill that builds a threat model for a target codebase. Two
modes: **bootstrap** derives the threat model from the target itself (source
tree, git history, public advisories, an optional past-vulns file);
**interview** discovers the threat model by walking an application owner
through the four-question framework. Both write `THREAT_MODEL.md` in a shared
schema.

## Status

The skill is read-only (it does not build,
run, or probe the target) and is safe to point at any local checkout. The
output is a starting point for human review, not a substitute for it.

## Why a threat model

Vulnerability scanners find instances; a threat model is the map of where
instances are likely to be and which ones matter. Hand a scanner a threat
model and it knows where to look. Hand triage a threat model and it knows
which findings to escalate. `/vuln-scan` and `/vuln-scan-diff` pick up
`THREAT_MODEL.md` automatically for their focus areas, and `/triage` uses it
to weigh findings.

## Model selection

The skill has no `model:` frontmatter pin; it runs on whatever model your
session uses (or `--model` if you pass one). It is designed for
reasoning-capable Claude models — use the same model you run the rest of the
pipeline with. If you want to lock the model regardless of session, add a
`model:` line to `SKILL.md`; frontmatter takes precedence over `/model` and
`--model`.

## Installation

Project-scoped (already done if you cloned this repo):

```bash
ls .claude/skills/threat-model/
```

User-scoped:

```bash
cp -r .claude/skills/threat-model ~/.claude/skills/
```

## Usage

### Bootstrap (derive from target, git history, advisories)

Use when no application owner is available. Point it at a checkout and,
optionally, a list of past vulnerabilities:

```
/threat-model bootstrap path/to/target
/threat-model bootstrap path/to/target --vulns path/to/target/vulns.txt
```

Without `--vulns` the skill mines `git log` and `CHANGELOG` itself; with it,
it ingests your supplied list first.

The skill spawns a parallel research swarm (docs reader, surface mapper, infra
reader, asset finder, git-history miner, and optionally a vuln-file parser),
synthesizes their returns into the system-context / assets / entry-points
sections, generalizes the collected vulns into threat classes, gap-fills with
STRIDE for surfaces the vuln history didn't cover, and writes
`path/to/target/THREAT_MODEL.md`. On small targets (<50 source files) it runs
the same briefs sequentially instead of spawning.

### Interview (discover via owner conversation)

Use when an application owner is in the session.

```
/threat-model interview path/to/target
/threat-model interview path/to/target --design-doc path/to/target/README.md
```

Without `--design-doc` the interview opens cold by asking the owner to
describe the system; with it, the skill reads the doc first and summarizes it
back for confirmation.

The skill will walk the owner through the four questions ("what are we working
on?", "what can go wrong?", "what are we going to do about it?", "did we do a
good job?"), grounding answers in the code as it goes, and write
`path/to/target/THREAT_MODEL.md`.

### Bootstrap then Interview (bootstrap a draft, then refine via interview)

Use when an owner is available but their time is limited: bootstrap produces
the draft unattended, then the interview spends owner time only on what the
code couldn't answer.

```
/threat-model bootstrap path/to/target/
/threat-model interview path/to/target/ --seed path/to/target/THREAT_MODEL.md
```

The interview will focus on the bootstrap's open questions instead of starting
cold.

## Checkpointing and resume (bootstrap mode)

Bootstrap writes per-stage checkpoints to `./.threat-model-state/` in the
current working directory (cwd-confined by `checkpoint.py`). If a run is
interrupted, re-invoking `/threat-model bootstrap <target-dir>` from the same
working directory
resumes from the last completed stage — the research swarm is not re-spawned
if Stage 1 already landed. Pass `--fresh` to start over. The state directory
is scratch; add it to `.gitignore`.

## Output

`<target-dir>/THREAT_MODEL.md` with seven sections: system context, assets,
entry points & trust boundaries, threats (the table), deprioritized, open
questions, provenance. See `schema.md` for the full contract.

## References

- Shostack, *The Four Question Framework for Threat Modeling* (2024) —
  https://shostack.org/files/papers/The_Four_Question_Framework.pdf
- OWASP Threat Modeling Cheat Sheet —
  https://cheatsheetseries.owasp.org/cheatsheets/Threat_Modeling_Cheat_Sheet.html
- This repo's `docs/security.md`.
