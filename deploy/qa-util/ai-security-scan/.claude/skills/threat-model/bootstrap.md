# /threat-model bootstrap

> **Re-read note:** If the Read tool reports "file unchanged" when you try to
> reload this file mid-session, the cached result was evicted. Reload with
> `cat .claude/skills/threat-model/bootstrap.md` via Bash instead.

Derive a threat model from **code and past vulnerability history** when no
application owner is available. Five stages: spawn a parallel research swarm,
synthesize its output into the first three sections of the schema plus a working
vuln table, cluster vulns into threat classes, gap-fill with STRIDE, emit
`THREAT_MODEL.md`.

This mode is read-only static analysis. It is language-agnostic: the same five
stages apply to C/C++, Rust, Go, Python, Java/Kotlin, JavaScript/TypeScript, or
any combination. Do not build, run, or probe the target. The Bash tool is
permitted only for `git` (history inspection), `find`/`ls` (layout), and `cat`
(reloading skill files when Read is evicted). Do not execute anything from
inside `<target-dir>`. Pass this restriction verbatim in every subagent prompt.

---

## Inputs

- `<target-dir>` (required): local checkout to analyse.
- `--vulns <path>` (optional): file of past vulnerabilities. Accepted formats:
  - Newline-separated CVE IDs (`CVE-2026-29022`)
  - CSV with columns `id,title,component,description` (extra columns ignored)
  - Markdown pentest report (headings + body parsed for finding descriptions)
  - JSON array of objects with at least `id` and `description` keys
- `--depth recon|full` (optional, default `full`): `recon` runs stages 1-2
  only. All eight schema sections are still written; sections 4, 5, and 8 are
  empty tables with a note "run with --depth full to populate". Use for quick
  context-building before a deeper pass.
- `--fresh` (optional): discard any existing checkpoint in
  `./.threat-model-state/` and restart from Stage 1.

If `--vulns` is absent, the vuln-file parser agent in the Stage-1 swarm is
skipped; the history miner agent covers the same ground from the repo's git log.

---

## Checkpointing

On large codebases the Stage-1 swarm can exhaust context before Stage 5 emits
the file. Each stage's output is persisted to `./.threat-model-state/` (relative
to the **current working directory**, not `<target-dir>`) so a resumed session
can skip completed stages.

All checkpoint reads and writes go through
`python3 .claude/skills/_lib/checkpoint.py` (atomic writes, JSON-validated). Do
not write `progress.json` directly with the Write tool. Never pass payloads via
heredoc or stdin — write payload to a `_chunk.tmp` file first, then call
checkpoint with `--from`. This keeps any target-derived bytes out of the Bash
argument list where they could collide with shell delimiters.

State files in `./.threat-model-state/`:
- `progress.json` — sole source of truth for the resume position:
  `{"status": "running"|"complete", "stage_done": N}`.
- `stageN.json` — data payload for stage N.
- `_chunk.tmp` — transient write buffer; overwritten before every checkpoint call.

**Beginning of run — resume check:**
```
python3 .claude/skills/_lib/checkpoint.py load ./.threat-model-state
```
- `status == "absent"` or `"complete"`, or `--fresh` is in `$ARGUMENTS` →
  fresh start: reset the directory, then proceed to Stage 1.
  ```
  python3 .claude/skills/_lib/checkpoint.py reset ./.threat-model-state
  ```
- `status == "running"` with `stage_done == N` → resume: read
  `stage1.json` through `stageN.json` in order, merging keys into working
  state (later files override earlier). Print `Resuming: Stage N complete`,
  then jump directly to Stage N+1.

**End of each stage N — two tool calls:**
1. Write tool → `./.threat-model-state/_chunk.tmp` with the stage output JSON.
2. Bash:
   ```
   python3 .claude/skills/_lib/checkpoint.py save ./.threat-model-state <N> <label> --key stage --from ./.threat-model-state/_chunk.tmp
   ```

**End of run:**
```
python3 .claude/skills/_lib/checkpoint.py done ./.threat-model-state 5 --key stage
```

---

## Stage 1 — Research swarm

Goal: gather in parallel everything needed to fill sections 1–3 of the schema
and build the working vuln table. Spawn the agents below **in a single batch**
with the Task tool so they run concurrently. Each agent receives a narrow brief,
the absolute path to `<target-dir>`, and the read-only constraint verbatim.

For small targets (fewer than 50 source files) or when `--depth recon` is set,
skip parallel spawning and run the briefs yourself sequentially — the overhead
is not worth it.

| Agent | Brief summary | Returns |
|---|---|---|
| **Docs reader** | Read `README*`, `SECURITY.md`, `CHANGELOG*`, top-level `docs/`, and the primary build manifest (`Cargo.toml`, `package.json`, `setup.py`, `CMakeLists.txt`, etc.). Summarise what the project says it is, who uses it, and any security claims or changelog fix entries. | Prose description; list of self-documented security fixes. |
| **Surface mapper** | Grep for entry-point signatures (table below). For each hit record: surface name, `file:function`, what crosses the boundary. Include supply-chain surfaces (lockfiles, vendored deps, `curl \| sh` in build scripts). Exclude `vendor/`, `node_modules/`, `third_party/`, generated code. Cap at ~5 representative hits per surface row. | Candidate section 3 rows: `{entry_point, description, trust_boundary, file_refs}`. |
| **Infra reader** | Read deploy configuration: `*.tf` / `*.tfvars`, k8s manifests under `k8s/` / `deploy/` / `manifests/`, `Dockerfile*`, CI workflow files, and IAM / service-account / ACL files. For each: (a) the identity the workload runs as and what it can reach, (b) any grant not managed in this tree, (c) credentials that survive a teardown or migration. | Candidate section 3 rows for infra surfaces, plus candidate section 4 rows where a configuration is itself a finding. |
| **Asset finder** | Identify what the code protects: secrets/keys, user records, databases, process integrity (always include for native code), service availability, and downstream embedder assets if the target is a library. | Candidate section 2 rows: `{asset, description, sensitivity}`. |
| **History miner** | (a) Glance at the build manifest and file extensions to identify language and domain. Derive 6–10 commit-message search terms specific to that stack, on top of the base set `CVE- security vuln fix exploit`. Derive from what the code does, not from a lookup table. (b) Run: `git -C <target-dir> log --all -i --grep='<terms, \|-joined>' --oneline`, then read the full message and diff of each hit. Also grep any `issues/` or `bugs/` export present in the tree. | Vuln rows: `{id (commit hash), title, component, class, vector}`. |
| **Vuln-file parser** | Only spawn if `--vulns <path>` was provided. Parse the file into normalised rows from any of the four accepted formats. | Vuln rows: `{id, title, component, class, vector}`. |

Surface-mapper grep targets — pass this table in the agent's prompt. Treat the
"Look for" column as a seed that sets the specificity bar; extend it with
idioms of the actual language and framework:

| Surface | Look for |
|---|---|
| Network | `listen`, `accept`, `bind`; HTTP route definitions; RPC / gRPC / GraphQL service registrations |
| File / format parsing | file-open calls; magic-byte checks; `parse` / `decode` / `load` / `unmarshal` function names |
| CLI / environment | argv parsers; `getenv` or equivalent |
| Deserialization | language-native deserializers on external data (`pickle`, `ObjectInputStream`, etc.) |
| DB / query | raw query-string construction; ORM `.raw()` / `.query()` escape hatches |
| IPC / plugins | `dlopen`; subprocess spawn; `eval` / `exec` on config; dynamic import |
| Supply chain | dependency lockfiles; vendored libraries; `curl \| sh` in build scripts |
| Infra / IAM | terraform `google_*_iam_*` / `aws_iam_*`; k8s `serviceAccountName`; secrets mounts |

**Stage 1 checkpoint payload:**
```json
{
  "stage": 1,
  "swarm": {
    "docs_reader": "<prose block>",
    "surface_mapper": [{"entry_point": "", "description": "", "trust_boundary": "", "file_refs": []}],
    "infra_reader": {"surfaces": [], "threats": []},
    "asset_finder": [{"asset": "", "description": "", "sensitivity": ""}],
    "history_miner": [{"id": "", "title": "", "component": "", "class": "", "vector": ""}],
    "vuln_file_parser": [{"id": "", "title": "", "component": "", "class": "", "vector": ""}]
  }
}
```

Agents that were skipped get an empty list or `null`. If the swarm ran inline
(small target), populate the same keys from your own sequential passes.

---

## Stage 2 — Synthesise

Goal: turn the Stage-1 outputs into sections 1–3 of the schema plus a working
vuln table. This stage runs in the orchestrating agent, not a subagent.

**Section 1: System context.** From the docs reader's summary plus a brief look
at the tree layout, write 1–2 paragraphs: what the system is, its language and
rough size, who deploys or embeds it, where it runs.

**Section 2: Assets.** Take the asset finder's rows. Remove duplicates; fill
obvious gaps (native code without "host process integrity" → add it). Assign
`sensitivity` per the scoring guide in `schema.md`.

**Section 3: Entry points and trust boundaries.** Merge surface mapper and infra
reader rows. Remove duplicates. For each entry point, name the trust boundary
crossing ("untrusted file → process memory", "unauthenticated HTTP → application
logic") and list which section 2 assets are reachable from it. Supply-chain,
build-time, and infra/IAM surfaces are entry points even though no runtime
input crosses them. **Every section 3 row must receive at least one section 4
threat** — this is the coverage invariant checked in Stage 5.

**Working vuln table.** Concatenate rows from history miner and vuln-file parser.
Deduplicate by `id`. For each row, determine which section 3 entry point it
traversed; read the relevant source to confirm. If a vuln's entry point is not
yet in section 3, the surface mapper missed it — add it now. Keep this table in
working notes; it does not appear in `THREAT_MODEL.md` verbatim. It becomes the
`evidence` column in Stage 3.

**Stage 2 checkpoint payload:**
```json
{
  "stage": 2,
  "section1_context": "<markdown prose>",
  "section2_assets": [{"asset": "", "description": "", "sensitivity": ""}],
  "section3_entry_points": [{"entry_point": "", "description": "", "trust_boundary": "", "reachable_assets": ""}],
  "vuln_table": [{"id": "", "title": "", "component": "", "class": "", "vector": "", "entry_point": ""}]
}
```

---

## Stage 3 — Cluster vulns into threat classes

Goal: group Stage-2 vulns into threat rows at the right abstraction level —
high enough that a threat survives any individual patch.

### 3a. Cluster

Group the vuln table by `(entry point, bug class, asset reached)`. Each cluster
becomes exactly **one** threat. Examples:
- Three heap overflows and one integer overflow, all in audio parsers, all
  reaching process memory → one threat: "Memory corruption leading to RCE via
  untrusted audio file parsing". Evidence: all four ids.
- Two SQL injections in different HTTP endpoints → one threat: "Data exfiltration
  and tampering via SQL injection in HTTP API". Evidence: both ids.

Apply the patch test to each cluster's threat statement: would the threat still
be true after every listed evidence item is patched? If not, zoom out further.

### 3b. Sibling scan (informs likelihood)

For each cluster, search for **siblings**: code paths with the same pattern that
were not in the vuln list (other format parsers, other endpoints calling the same
unsafe helper, other size fields multiplied without overflow checks). You are not
proving these are exploitable; you are estimating how much of the surface shares
the same shape. More siblings → higher likelihood.

Keep sibling locations in working notes and surface them in the Stage 5
hand-back as candidate leads for `/vuln-scan`. Do **not** put sibling `file:func`
references in the section 4 `evidence` column — that column is for confirmed past
vulnerabilities only.

### 3c. Score

For each cluster, assign:

- `actor`: derived from the entry point (file parsing → whoever supplies the
  file; network endpoint → `remote_unauth` or `remote_auth` depending on whether
  authentication precedes it).
- `impact`: from the asset and bug class (memory corruption on a network service
  → `critical`; non-sensitive info leak → `low`).
- `likelihood`: from the evidence. One confirmed past vuln on this exact surface
  → at least `likely`. Public exploit or active exploitation → `almost_certain`.
  No evidence but siblings found and the technique is well known → `possible`.
  Adjust down for existing controls.
- `controls`: grep for relevant mitigations (input validation, size caps,
  sandboxing, ASLR/CFI for native code, parameterised queries, auth middleware,
  rate limiting, etc.). `none` if none found.
- `status`: `unmitigated` unless you found a control that fully closes the
  threat.
- `recommended_mitigation` (working notes only, not a section 4 column): one
  class-level control that would close or materially reduce the entire threat
  cluster, regardless of which specific instance is found next (e.g., "sandbox
  the decoder process", "parameterised queries everywhere", "size-cap all length
  fields before allocation"). These become section 8 rows in Stage 5.

**Stage 3 checkpoint payload:**
```json
{
  "stage": 3,
  "section1_context": "...",
  "section2_assets": [],
  "section3_entry_points": [],
  "section4_threats": [{"threat": "", "actor": "", "surface": "", "asset": "", "impact": "", "likelihood": "", "status": "", "controls": "", "evidence": ""}],
  "mitigation_notes": [{"cluster": "", "recommended_mitigation": ""}],
  "sibling_locations": [{"threat": "", "locations": []}]
}
```

---

## Stage 4 — STRIDE gap-fill

Past vulnerabilities are biased toward what has already been found. A threat
model must also cover what has not been found yet. For **every section 3 entry
point that currently has no section 4 row**, walk STRIDE and add any plausible
threats:

| Letter | Ask for this entry point |
|---|---|
| Spoofing | Could an attacker impersonate a trusted source? |
| Tampering | Could data be modified in transit or at rest? |
| Repudiation | Could an action occur without attributable logs? |
| Information disclosure | Could data be read that should be protected? |
| Denial of service | Could a resource (CPU, memory, disk, connections) be exhausted? |
| Elevation of privilege | Could an attacker end up with more access than they started with? |

Also revisit entry points that **do** have existing rows: is the current row the
only plausible threat, or do other STRIDE categories also apply? (A file parser
with an RCE threat likely also has a DoS threat.)

For **infra and IAM entry points**, STRIDE maps less cleanly. Walk these
questions instead:
- **Over-grant**: does the identity reach more than the application needs?
- **Lateral identity**: can a co-located workload assume this identity?
- **Drift**: is any grant managed outside this tree and therefore never reviewed
  or torn down with the code?
- **Residual access**: do credentials from a predecessor system survive the
  migration?
- **Column exposure**: does a broad table read expose columns the app does not
  need?
- **Scope enforcement**: what bounds an automated write or merge path to its
  intended scope?

Gap-fill threats have empty `evidence` — this is expected. Score `likelihood`
from technique prevalence and surface reachability alone. **The final section 4
table must contain at least one row with empty `evidence`**, confirming this
stage ran.

Populate `## 5. Deprioritized` with STRIDE categories you considered and ruled
out, with the reason (e.g., "Repudiation: not applicable, no multi-user actions").

**Stage 4 checkpoint payload:**
```json
{
  "stage": 4,
  "section1_context": "...",
  "section2_assets": [],
  "section3_entry_points": [],
  "section4_threats": [],
  "section5_deprioritized": [{"threat": "", "reason": ""}],
  "mitigation_notes": [],
  "sibling_locations": []
}
```

---

## Stage 5 — Emit

**Coverage check (do this before writing the file).** For every section 3 entry
point, confirm that at least one section 4 row names it in the `surface` column.
Match on the exact entry-point name string. If any section 3 row has no
corresponding section 4 coverage, return to Stage 4 and add the missing threat.

Sort section 4 by (impact descending, likelihood descending). Assign `id` = `T1`,
`T2`, … in sorted order.

Populate `## 6. Open questions` with everything the static analysis could not
determine:
- Deployment context ("Is this exposed to the network, or only internal?")
- Intended actors ("Who supplies input files in practice?")
- Controls you could not verify ("Is there a WAF or size limit upstream?")
- Risk appetite ("Is denial of service acceptable for this use case?")

These questions seed a later `/threat-model interview --seed THREAT_MODEL.md` pass.

Populate `## 8. Recommended mitigations` from the Stage-3c working notes: one
row per class-level control, listing the `threat_ids` it covers, `closes_class`
(yes/partial), and rough `effort` (S/M/L). If two clusters are closed by the
same control, emit one row covering both. Gap-fill threats from Stage 4 get
mitigation rows where an obvious class-level control exists.

**Write the file incrementally.** Assemble in `./.threat-model-state/THREAT_MODEL.md`
one section at a time (a stalled write loses only that section, not the file),
then copy the assembled result to `<target-dir>/THREAT_MODEL.md`.

1. Write tool → `./.threat-model-state/THREAT_MODEL.md` (clobbers any prior
   file) with only the title line and `## 1. System context`.
2. For each remaining section in schema order:
   - Write tool → `./.threat-model-state/_chunk.tmp` with that section's markdown.
   - Bash:
     ```
     python3 .claude/skills/_lib/checkpoint.py append ./.threat-model-state/THREAT_MODEL.md --from ./.threat-model-state/_chunk.tmp
     ```
3. Read `./.threat-model-state/THREAT_MODEL.md`, then Write tool →
   `<target-dir>/THREAT_MODEL.md` with the same content.

Set the `## 7. Provenance` section to:
```
- mode: bootstrap
- date: <today>
- target: <target-dir> @ <git -C <target-dir> rev-parse --short HEAD, or "not a git repo">
- inputs: <--vulns path, or "git log mined">
- owner: unset
```

**Final checkpoint:**
```
python3 .claude/skills/_lib/checkpoint.py done ./.threat-model-state 5 --key stage
```

Hand back to the user:
1. Path to the written file.
2. Top 5 threats by impact × likelihood (id, threat, scores).
3. Count of threats with evidence vs. without (confirms gap-fill ran).
4. Stage-3b sibling locations as candidate focus areas for `/vuln-scan`.
5. Top 3 section 8 mitigations by (closes_class first, then effort ascending).
6. Section 6 open questions, framed as "ask the owner in a follow-up interview".
