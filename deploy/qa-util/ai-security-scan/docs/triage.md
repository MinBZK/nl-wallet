# Triage: "How do I go through these hundreds of findings?"

A scan (ours or another scanner's) just produced a pile of raw findings.
The `/triage` skill turns that pile into a short, ranked, owned list that
engineering can act on.

## What it does

The skill does four things in a single pass:

1. **Verify.** Adversarially checks each finding against the source code
   (read-only, does not execute code), and drops the ones that aren't real.
2. **Deduplicate.** Collapses the same root cause reported N times across
   parallel runs or multiple scanners.
3. **Re-rank.** Derives severity from preconditions and your stated trust
   boundary. For example, a "HIGH" behind one or two preconditions and
   authenticated access becomes a MEDIUM.
4. **Route.** Tags each survivor with a component owner so it can be routed
   appropriately.

It outputs `TRIAGE.md` (a human-readable, ranked list of findings) and
`TRIAGE.json` (a machine-readable list of findings, for your tracker or
other downstream use).

## The rules it applies

- **Duplicates.** Two findings are duplicates if fixing one fixes the other.
  The skill attempts to identify those cases using two passes. First, a
  cheap deterministic pass that checks if two findings are in the same file,
  have the same category, and reference line numbers within ten lines. Second,
  an LLM pass that asks the model to use semantic reasoning to identify
  duplicates.
- **Severity.** Based on what an attacker would actually have to do to exploit
  the finding. The verifier lists preconditions first, then maps the count to
  a score - none, with unauthenticated remote access = High; one or two, or an
  authenticated path = Medium; three or more, or local-only = Low. You can swap
  in your own scoring standard when the skill asks at the start of a run.

## Run it

```bash
# On /vuln-scan output
> /triage ./VULN-FINDINGS.json --repo ./path/to/source

# On /vuln-scan-diff output
> /triage ./VULN-FINDINGS-DIFF.json --repo ./path/to/source

# Non-interactive, with more verifier votes per finding (default is 3)
> /triage ./findings/ --auto --votes 5 --repo ./path/to/source
```

By default, the skill **interviews you first** about your trust boundary,
your threat model, your scoring standard (HIGH/MED/LOW vs. CVSS vs. your org bug-bar),
and whether to bias toward precision or recall on split votes. These answers shape
verification and ranking. Pass `--auto` to skip the interview and use
precision-biased defaults — that is what the CI jobs do.

## When to use triage

Always, before spending engineering time on scan output. `/vuln-scan` and
`/vuln-scan-diff` intentionally bias toward recall, so their raw findings
contain false positives; `/triage` is the layer that removes them, and it
works on *any* findings file — a fresh scan, overlapping results from
several runs, or an old backlog from other tools.

## After triage

Confirmed findings (`TRIAGE.md`, "Act on these") go to the component owner
named in `owner_hint`. Findings marked `needs_manual_test` need a human to
build a controlled proof of concept — static reasoning alone could not
settle them.
