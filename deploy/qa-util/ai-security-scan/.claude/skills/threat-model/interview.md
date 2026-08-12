# /threat-model interview

> **Re-read note:** If the Read tool reports "file unchanged" when you try to
> reload this file mid-session, the cached result was evicted. Reload with
> `cat .claude/skills/threat-model/interview.md` via Bash instead.

Build a threat model through a **structured owner interview**. The owner is
present in the session; your role is to ask, listen, ground answers in code
where possible, and emit `THREAT_MODEL.md` per `schema.md`.

The four questions (introduce each phase using this exact wording):

1. **What are we working on?**
2. **What can go wrong?**
3. **What are we going to do about it?**
4. **Did we do a good job?**

---

## Inputs

- `<target-dir>` (required): local checkout. Read it to corroborate owner
  answers; do not execute anything inside it.
- `--design-doc <path>` (optional): architecture or design document. Read this
  before Q1 so you can summarise back to the owner rather than asking cold.
- `--seed <THREAT_MODEL.md>` (optional): a prior bootstrap output. When
  present, focus the interview on its `## 6. Open questions` and any threats
  with uncertain scores, rather than building from scratch.

---

## Tracking claim provenance

Every fact you write carries one of two tags in your working notes:

- `[Code-verified]` — you read the source and confirmed the claim.
- `[Owner-states]` — the owner asserted it and you have not verified it in code.

The final `THREAT_MODEL.md` does not include these tags inline, but every
`[Owner-states]` fact that affects a likelihood or status score must appear in
`## 6. Open questions` as a follow-up item to verify. This keeps the model
honest about what was observed versus asserted.

---

## Method

Work through the four questions in order. Within each question, ask one thing
at a time and wait for the answer before continuing. Do not present a
questionnaire all at once.

---

### Q1 — What are we working on?

**Goal:** fill `## 1. System context`, `## 2. Assets`, `## 3. Entry points & trust boundaries`.

If `--design-doc` was provided: read it, then **summarise the system back to
the owner in 4–6 sentences** and ask "Is this right? What did I miss?" This
surfaces drift between documented and actual design more efficiently than asking
from scratch.

If no design doc: ask these prompts in order:
- "In two or three sentences, what does this system do and who uses it?"
- "What data does it hold or pass through that would be bad to lose, leak, or
  tamper with?" → assets table.
- "Where does input come from? Walk me from the outside in: network, files,
  CLI, other services — anything a user or another system hands you." → entry points.
- "Where does trust or privilege level change — for example, unauthenticated to
  authenticated, user to admin, or one service trusting another?" → trust boundaries.

While the owner answers, **read the code** to corroborate: look for `main`,
route definitions, file-open calls, socket listeners, deserialisers, and
argument parsing. Tag confirmed facts `[Code-verified]`. If the code shows an
entry point the owner did not mention, ask about it: "I see an `/admin/debug`
route in `routes.py:88` — is that reachable in production?"

If `--seed` was provided: read its sections 1–3, summarise back, and ask only
"What's wrong or missing here?"

---

### Q2 — What can go wrong?

**Goal:** fill `## 4. Threats` rows — id, threat, actor, surface, asset.

Open broadly: **"For each of those entry points, what can go wrong? What is the
worst thing someone could do?"** Let the owner answer in their own words first.
Capture each answer as a candidate threat row.

When the owner stalls or stays vague, use structured STRIDE prompts. Walk each
section 3 entry point:

| Letter | Ask |
|---|---|
| Spoofing | "Could someone pretend to be a user or service they are not, at this point?" |
| Tampering | "Could input or stored data be modified in transit or at rest?" |
| Repudiation | "If something bad happened here, would you know who did it?" |
| Information disclosure | "Could this leak data it should not?" |
| Denial of service | "Could someone make this unavailable or unacceptably expensive to run?" |
| Elevation of privilege | "Could someone end up with more access than they started with?" |

Then derive the domain-specific threat classes most relevant to this system.
From section 1 (stack, language, deployment, data flows), name 5–8 classes at
concrete granularity — "IDOR on dataset rows" or "integer overflow on length
fields", not "web vulnerabilities" or "memory bugs". Derive from what the owner
described, not from a generic checklist.

Show the derived list: "Based on what you've described, these are the classes I'd
focus on. Anything you'd add from incidents you've seen on this or similar
systems?" Weight owner additions highly — they reflect institutional memory.
If a class you would expect for this stack (injection, deserialisation, auth,
memory safety, crypto, supply chain, infra/IAM) is absent from both lists, ask
why before dropping it.

Walk each section 3 entry point through STRIDE and the confirmed class list. For
each candidate threat: identify the **actor** (from the enum in `schema.md`),
**surface** (which section 3 entry point), and **asset** (which section 2 row).
Phrase the threat at the level where it survives any single patch.

If `--seed` was provided: go through the seed's section 4 table row by row and
ask "Does this apply, and is the actor right?" Then ask "What's missing?"

---

### Q3 — What are we going to do about it?

**Goal:** fill `impact`, `likelihood`, `status`, `controls` for every section 4
row, and populate `## 5. Deprioritized`.

For each threat row, ask:

- "What is in place today that stops or limits this?" → `controls`. Verify in
  code where possible (`[Code-verified]` vs `[Owner-states]`).
- "If it happened anyway, how bad would it be?" → `impact` (read the impact
  scale from `schema.md` if the owner wants reference).
- "How likely is it that someone would try and succeed, given the controls?" →
  `likelihood`. If past incidents, CVEs, or pentest findings exist for this
  surface, list them in `evidence` and weight likelihood upward.
- "Is this mitigated, partially mitigated, unmitigated, or accepted?" → `status`.
  If the owner says "risk accepted", record their reason verbatim and move the
  row to `## 5. Deprioritized`.

Not having a mitigation is a valid outcome. "We are not going to address this,
and here is why" is a threat model decision, not a failure.

After scoring, ask one closing question **per threat class** (not per row):
"If you could land one engineering control that makes this entire class go away
or shrink, what would it be?" Record the answer — or your own proposal if the
owner defers — as a section 8 row: `mitigation | threat_ids | closes_class |
effort`. Prefer controls that survive the next bug (sandboxing, type-safe
parsers, parameterised queries, CSP, allocation caps) over patches for the
most recent one.

---

### Q4 — Did we do a good job?

**Goal:** validate the model before writing it.

- Read the draft section 4 table back to the owner, sorted by impact × likelihood.
  Ask: **"Does the top of this list match your intuition? Is anything ranked too
  high or too low?"** Adjust based on feedback.
- Ask: **"Is there anything you have been worried about that is not on this list?"**
  Add anything new.
- Check coverage: for every section 3 entry point, the `entry_point` name must
  appear verbatim in at least one section 4 `surface` cell, or a section 5 row
  must say "<entry_point>: out of scope because …". If neither, either add a
  threat for that surface or ask the owner why it is safe and record the answer
  in section 5.
- Ask: **"Would you do this again for the next service? What would make it
  easier?"** Record the answer in your hand-back to the user (not in the file);
  it is feedback for this skill.

---

## Emit

Write `<target-dir>/THREAT_MODEL.md` per `schema.md`. Set `## 7. Provenance`:

```
- mode: interview
- date: <today>
- target: <target-dir> @ <git -C <target-dir> rev-parse HEAD, if available>
- inputs: <design-doc path, or "none">; <seed path, or "none">
- owner: <name the user gave, or "present, unnamed">
```

Hand back to the user:

1. Path to the written file.
2. Top 5 threats by impact × likelihood, one line each.
3. Top 3 section 8 mitigations by (closes_class first, then effort ascending).
4. Every `[Owner-states]` claim that affects a score, formatted as section 6
   bullets: `- [Owner-states] <claim>. Affects: <Tn field>. Verify by: <check>.`
5. If `--seed` was provided: a short diff summary — what was added, what changed,
   what the owner corrected from the bootstrap draft.
