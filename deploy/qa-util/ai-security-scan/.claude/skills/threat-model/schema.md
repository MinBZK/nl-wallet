# THREAT_MODEL.md schema

> **Re-read note:** If the Read tool reports "file unchanged" when you try to
> reload this file mid-session, the cached result was evicted. Reload with
> `cat .claude/skills/threat-model/schema.md` via Bash instead.

Both `/threat-model bootstrap` and `/threat-model interview` write
`<target-dir>/THREAT_MODEL.md` in this format. The file is human-readable
markdown, but the section headings, table columns, and enum values below are a
contract: keep headings and column order exactly as shown so downstream tooling
can locate sections with a simple regex.

---

## Required sections, in order

```markdown
# Threat Model: <system name>

## 1. System context

## 2. Assets

## 3. Entry points & trust boundaries

## 4. Threats

## 5. Deprioritized

## 6. Open questions

## 7. Provenance

## 8. Recommended mitigations
```

A consumer that only needs the threat table can match `^## 4\. Threats$` and
read until the next `^## `. Section 8 is optional and additive — older threat
models may omit it; consumers must tolerate its absence.

---

## Section contents

### 1. System context

One to three paragraphs of prose describing what the system is, what it does,
who uses it, and where it runs. No table. This is the answer to "what are we
working on?"

### 2. Assets

Markdown table. One row per thing worth protecting.

| asset | description | sensitivity |
|---|---|---|

`sensitivity` ∈ {`low`, `medium`, `high`, `critical`}.

### 3. Entry points & trust boundaries

Markdown table. One row per place where untrusted input enters the system or
where privilege level changes.

| entry_point | description | trust_boundary | reachable_assets |
|---|---|---|---|

`trust_boundary` is free text naming the crossing (e.g. "untrusted file →
process memory", "unauthenticated network → authenticated session").
`reachable_assets` is a comma-separated list of asset names from section 2.

### 4. Threats

Markdown table. **This is the threat model proper.** One row per
actor-wants-outcome pair, stated at an abstraction level where the threat
remains valid after any individual bug is patched.

| id | threat | actor | surface | asset | impact | likelihood | status | controls | evidence |
|---|---|---|---|---|---|---|---|---|---|

- `id`: `T1`, `T2`, … Stable identifiers — do not renumber when rows are
  removed.
- `threat`: One sentence, active voice, naming the outcome. Example:
  "Remote code execution via untrusted media parsing", not "buffer overflow in
  dr_wav".
- `actor` ∈ {`remote_unauth`, `remote_auth`, `adjacent_network`,
  `local_user`, `local_admin`, `supply_chain`, `insider`}.
- `surface`: Which entry point(s) from section 3 this threat traverses.
- `asset`: Which asset(s) from section 2 this threat compromises.
- `impact` ∈ {`low`, `medium`, `high`, `critical`, `existential`}.
- `likelihood` ∈ {`very_rare`, `rare`, `possible`, `likely`,
  `almost_certain`}.
- `status` ∈ {`unmitigated`, `partially_mitigated`, `mitigated`,
  `risk_accepted`}.
- `controls`: Current mitigations in plain text, or `none`.
- `evidence`: CVE IDs, commit hashes, issue links, or pentest finding IDs that
  instantiate this threat. May be empty. Evidence raises likelihood; it is not
  the threat itself.

Sort by (impact, likelihood) descending so the top rows are the priorities.

### 5. Deprioritized

Markdown table. Threats that were considered and explicitly parked.

| threat | reason |
|---|---|

Common reasons: out of scope, actor not in threat model, asset not present,
risk accepted by owner.

### 6. Open questions

Bullet list. Things the mode could not determine. For `bootstrap`, these are
questions for a human owner; for `interview`, these are claims the owner made
that were not verifiable in the code.

### 7. Provenance

```markdown
- mode: interview | bootstrap | bootstrap-then-interview
- date: YYYY-MM-DD
- target: <path or repo url @ commit>
- inputs: <design doc path | --vulns path | "none">
- owner: <name, for interview mode> | <unset, for bootstrap>
```

### 8. Recommended mitigations

Optional. Each row is one **class-level control** — a mitigation that closes or
materially reduces an entire threat cluster regardless of which specific instance
is found next. Not a per-finding patch list.

| mitigation | threat_ids | closes_class | effort |
|---|---|---|---|

- `mitigation`: Imperative, one line (e.g., "sandbox the decoder process",
  "parameterised queries everywhere", "size-cap all length fields before
  allocation").
- `threat_ids`: Comma-separated section 4 ids this mitigation covers (e.g.,
  `T1,T3`).
- `closes_class`: `yes` | `partial`.
- `effort`: `S` | `M` | `L`.

---

## Scoring guide

### Impact

| value | meaning |
|---|---|
| `low` | Nuisance; no data or availability loss. |
| `medium` | Limited data exposure or degraded availability for some users. |
| `high` | Significant data exposure, integrity loss, or full availability loss. |
| `critical` | Full compromise of a primary asset — RCE, auth bypass, data exfiltration at scale. |
| `existential` | Compromise threatens the organisation's continued operation. |

### Likelihood

| value | meaning |
|---|---|
| `very_rare` | Requires nation-state resources or an unlikely chain of preconditions. |
| `rare` | Requires significant skill and a non-default configuration. |
| `possible` | A motivated attacker with public tooling could plausibly execute this. |
| `likely` | The surface is reachable, the technique is well known, and prior evidence exists in this or similar systems. |
| `almost_certain` | Actively exploited in the wild, or trivially automatable against the default configuration. |

Evidence (past CVEs on the same surface, pentest findings, public exploit code)
moves likelihood **up**. Existing controls move it **down**. Score the
**residual** likelihood after current controls are applied.

---

## Example (excerpt)

```markdown
## 4. Threats

| id | threat | actor | surface | asset | impact | likelihood | status | controls | evidence |
|---|---|---|---|---|---|---|---|---|---|
| T1 | Memory corruption leading to RCE via untrusted audio parsing | remote_unauth | WAV/FLAC decoders | host process integrity | critical | likely | unmitigated | none | CVE-2026-29022, CVE-2025-14369 |
| T2 | Denial of service via resource exhaustion during decode | remote_unauth | FLAC decoder | service availability | medium | likely | unmitigated | none | CVE-2025-14369 |
| T3 | Supply-chain compromise of vendored single-header dependency | supply_chain | build pipeline | host process integrity | critical | rare | partially_mitigated | pinned commit | |
```

T1 remains in the model after both CVEs are patched. Attackers will still send
malformed audio files; the CVEs are evidence the surface is fertile, not the
threat itself.
