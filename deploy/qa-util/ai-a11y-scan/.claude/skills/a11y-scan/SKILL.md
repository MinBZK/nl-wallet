---
name: a11y-scan
description: >-
  AI accessibility scan of NL Wallet screen captures against WCAG 2.1 AA.
  Reads a folder of Appium page-source XML + screenshot pairs (one per
  screen/platform), spawns one review subagent per screen, applies the
  false-positive suppression file, and writes an axe-form A11Y-REPORT.json +
  .md. Read-only — it inspects captures, never builds or drives the app. Use
  when asked to "scan for accessibility issues", "run the a11y scan", "check
  WCAG on these screens", or to process captures from captureA11ySnapshot.
argument-hint: "<captures-dir> [--platform android|ios] [--screen <name>] [--no-best-practice] [--fp-rules <file>] [--no-suppress] [--single]"
allowed-tools:
  - Read
  - Glob
  - Grep
  - Write
  - Edit
  - Task
  - Bash(rg:*)
  - Bash(grep:*)
  - Bash(ls:*)
  - Bash(wc:*)
  - Bash(head:*)
  - Bash(file:*)
  - Bash(find:*)
  - Bash(cat:*)
  - Bash(jq:*)
  - Bash(echo:*)
  - Bash(cd:*)
  - Bash(python3:*)
  - Bash(sips:*)
---

# /a11y-scan

Accessibility review of NL Wallet screen captures against **WCAG 2.1 AA**.
Produces `A11Y-REPORT.json` — a wrapper holding one [axe-core](https://github.com/dequelabs/axe-core)
result object per screen — plus a human-readable `A11Y-REPORT.md`.

**No app execution.** This skill reads the page-source XML and screenshots the
e2e tests already collected and reasons about them. It never builds, launches,
or drives the app; live-only criteria (focus visibility, keyboard operability,
reflow, orientation) are out of scope — see `wcag21aa.md`.

**The ruleset lives in `wcag21aa.md`** (next to this file). Read it in full
before fanning out and pass it verbatim to every subagent — it covers all 50
WCAG 2.1 A/AA success criteria (a coverage map plus detailed active rules) and
defines every check's axe rule id, WCAG tags, impact, and how to detect it from
Android and iOS page source plus the screenshot. Its coverage map is derived
from `wcag21-AA-full.json` (the W3C spec export, also in this directory); that
JSON is the authoritative criterion list and does not need to be sent to
subagents — `wcag21aa.md` is the working ruleset.

**Tool fallbacks.** Prefer the built-in Glob, Grep, and Read (Read renders PNG
screenshots visually). When Glob/Grep are unavailable, use the Bash fallbacks
in `allowed-tools`. Do not write helper scripts or pipe capture content into a
shell interpreter.

## Arguments

- `<captures-dir>` (required) — directory of capture pairs. Each screen is a
  `<screen>.<platform>.xml` page source and a `<screen>.<platform>.png` (or
  `.jpg`) screenshot, as written by `MobileActions.captureA11ySnapshot`.
- `--platform android|ios` — restrict to one platform (repeatable-free; default
  both).
- `--screen <name>` — restrict to captures whose screen name matches (substring,
  repeatable).
- `--no-best-practice` — drop the `best-practice`-tagged rules (target size,
  tap spacing). They **run by default**; pass this to keep the report strictly
  WCAG 2.1 AA.
- `--fp-rules <file>` — false-positive rules file to apply during suppression
  (default: `false-positives.md` in this component's root, resolved relative to
  the skill).
- `--no-suppress` — skip suppression entirely; report every finding raw.
- `--single` — skip subagent fan-out; evaluate screens sequentially in one pass.
  Useful for a handful of screens or debugging.

## Step 1 — Scope

1. Resolve `<captures-dir>`. If it does not exist or contains no `.xml` files,
   stop with a clear error naming the path.
2. Enumerate captures. Ignore `._*` and `.DS_Store` — archives zipped on macOS
   carry an AppleDouble shadow of every entry, and `._<screen>.<platform>.xml`
   pairs with a `._<screen>.<platform>.png`, so it looks like a capture without
   being one. For every remaining `*.xml`, parse the trailing name segments:
   the segment before the extension is the **platform** (`android` | `ios`);
   everything before it is the **screen name**. Pair it with the screenshot of
   the same base name (`.png` then `.jpg`). Record pairs missing a screenshot
   (evaluate XML-only, note the missing image) and screenshots missing an XML
   (skip — the page source is required).
3. Apply `--platform` and `--screen` filters. If nothing remains, stop and say so.
4. Read the device metadata. For each platform present, glob
   `<platform>.*.device.json` — two per platform, one per test run
   (`android.browserstack.device.json`, `android.local-device.device.json`) —
   and take `scale`, the divisor that turns raw geometry into dp (Android) or pt
   (iOS). Android `bounds` are device pixels, so `target-size` is meaningless
   without it. If no file carries a positive numeric `scale`, or the files
   disagree, record the scale as **unavailable** for that platform; never
   substitute a default density or pick one of several.
5. Read the capture provenance. If `sources.json` exists in the captures dir,
   read it: `scan_commit` is the commit of the job running the scan, and each
   `<platform>.commit` is the commit the *captures* for that platform came
   from. They routinely differ — the archives sit at a fixed path and are
   refreshed per platform by whichever e2e run last succeeded, so Android and
   iOS captures can be from different commits, and both can predate
   `scan_commit`. Carry the values through to the report verbatim; never
   substitute `scan_commit` for a missing platform commit. If the file is
   absent (a local run) or a platform is missing from it, record that
   platform's provenance as `unknown`.
6. Read `wcag21aa.md`. Select the active rule set: the Level A/AA rules always,
   plus the `best-practice` rules by default; drop the `best-practice` rules
   only if `--no-best-practice` is given.
7. Report the capture count, the platform split, the per-platform scale with
   the environments it was read from (or that it is unavailable and why, and
   that target size will come back `incomplete`), the active rule ids, and any
   unpaired captures before fanning out.

## Step 2 — Fan out

Unless `--single`, spawn **one Task subagent per screen capture** concurrently
(`subagent_type: "general-purpose"`, `description: "a11y {screen}.{platform}"`),
capped at 10 in flight. If there are more than 10 captures, shard into
sequential batches of ≤ 10, each batch a single message. For 3 or fewer
captures, fall through to `--single` automatically.

Each subagent receives the brief below with its capture and the full ruleset
filled in.

### Review brief (per subagent)

```
You are conducting an authorized accessibility review of one NL Wallet app
screen against WCAG 2.1 AA. Other agents review other screens.

SCREEN: {screen}
PLATFORM: {android | ios}
PAGE SOURCE (Appium XML): {abs path to .xml}   ← Read this file
SCREENSHOT: {abs path to .png, or "none — evaluate from page source only"}
                                                ← Read this file to view the pixels
GEOMETRY SCALE: {scale, e.g. 3.5 — divide raw px/pt sizes by this for dp/pt
                 | "unavailable ({reason}) — report target-size as incomplete"}
ACTIVE RULES: {"Level A/AA plus best-practice" | "Level A/AA only"}

RULESET — apply exactly these checks; each entry defines the rule id, WCAG
tags, impact, and how to detect the issue from Android/iOS page source and the
screenshot:
--- begin wcag21aa.md ---
{verbatim contents of wcag21aa.md}
--- end wcag21aa.md ---

METHOD:
1. Read the page source XML. Build a picture of the visible, on-screen
   elements (skip non-visible / off-screen nodes per the ruleset).
2. Read the screenshot to see the rendered result. Use it for every
   colour/contrast rule and to tell informative imagery from decorative.
3. Walk each active rule over the elements. For every issue, locate the
   responsible element in the page source and capture a locator and snippet.

REPORTING BAR — aggressive (recall over precision): report a
status=violation whenever a rule's condition is met on a visible, in-scope
element. Being unsure the issue *matters* is NOT a reason to soften to
incomplete — flag it and note the uncertainty in the description; a human (or
false-positive suppression) triages later. Reserve status=incomplete for the
narrow case where you cannot read the **evidence itself** — contrast colours
that cannot be sampled from the screenshot (gradient/photo/overlap) or a node
the page source does not describe enough to judge. "Might not be a problem" =
violation; "cannot measure this" = incomplete. Do NOT report the global
exclusions in the ruleset, disabled controls for contrast, or the "not
statically checkable" criteria.

For contrast findings, read the actual foreground and background colours from
the pixels around the text/component in the screenshot and compute the ratio.
Never invent a ratio you did not derive from the image.

OUTPUT — one XML block per finding, nothing else:

<finding>
<rule_id>{axe-analogous id from the ruleset, e.g. image-alt | label | structure | reading-order | sensory-characteristics | identify-input-purpose | use-of-color | color-contrast | image-of-text | non-text-contrast | screen-title | link-name | descriptive-headings-labels | label-in-name | error-identification | error-suggestion | button-name | control-role-state | target-size}</rule_id>
<status>{violation | incomplete}</status>
<wcag_sc>{e.g. 1.1.1}</wcag_sc>
<tags>{comma-separated, e.g. cat.text-alternatives,wcag2a,wcag111,wcag21aa}</tags>
<impact>{critical | serious | moderate | minor}</impact>
<target>{a locator for the element: Android resource-id or content-desc or an xpath; iOS name or type+index}</target>
<element>{the page-source element snippet, trimmed to one node}</element>
<description>{what is wrong and why it fails the criterion}</description>
<failure_summary>{"Fix any of the following:" + the concrete fix, axe-style}</failure_summary>
<data>{for contrast: fg/bg hex, computed ratio, required ratio, size class; else omit}</data>
<confidence>{0.0-1.0}</confidence>
</finding>

If the screen has no issues after a thorough pass, emit one <finding> with
rule_id=none and a one-line note of what was reviewed.
```

## Step 3 — Collate

1. Collect every `<finding>` block from all subagents. Drop `rule_id=none`
   placeholders (but keep a note that the screen was clean).
2. **Light dedupe within a screen:** if two findings share the same `rule_id`
   and the same `target`, keep the one with the longer description.
3. Group findings by screen (`{screen}.{platform}`). Split each screen's
   findings into `violations` (status=violation) and `incomplete`
   (status=incomplete).

## Step 4 — Suppress false positives (skip with `--no-suppress`)

Apply the false-positive rules so accepted findings do not resurface every run.

1. Read the `--fp-rules` file (default `false-positives.md` in the component
   root). If it is missing, warn and continue with no suppression.
2. For each `violation` (suppression does not touch `incomplete`), decide
   whether it matches an accepted rule. Match on **screen + rule class + element**
   as the rule text describes — never on finding ids. When there are more than
   ~20 violations, spawn one Task subagent to do the matching with this prompt;
   otherwise match inline:

```
You are applying accepted-false-positive rules to accessibility findings.
A finding is SUPPRESSED only if it clearly falls within the scope of a rule
below — the same screen/component AND the same class of finding the rule
accepts. When in doubt, do NOT suppress.

RULES:
{verbatim contents of the fp-rules file}

FINDINGS (id | screen | rule_id | target | one-line description):
{list}

Respond ONLY with lines:
  SUPPRESS: <finding_id> | rule <N> — <short reason>
Omit findings that are not suppressed. No prose.
```

3. Move each suppressed finding out of `violations` into a top-level
   `suppressed[]` entry recording the finding, the matched rule number, and the
   reason. `violations` keeps only what survived.

## Step 5 — Build the axe-form report

Assign stable ids `A-001`, `A-002`, … across all surviving findings ordered by
(impact desc: critical>serious>moderate>minor, then screen, then rule_id).

Build one **axe result object per screen** and wrap them:

**`A11Y-REPORT.json`** (written to this component's root)

```json
{
  "wcag_version": "wcag21aa",
  "generated_at": "<iso8601>",
  "captures_dir": "<captures-dir>",
  "capture_sources": {
    "scan_commit": "<commit of the job that ran the scan, or \"unknown\">",
    "android": { "commit": "<commit the Android captures came from>", "archive": "<minio path>" },
    "ios": { "commit": "<commit the iOS captures came from>", "archive": "<minio path>" }
  },
  "rules_run": ["image-alt", "button-name", "..."],
  "results": [
    {
      "testEngine": { "name": "nl-wallet-ai-a11y-scan", "version": "1.0.0" },
      "testRunner": { "name": "claude-code" },
      "testEnvironment": { "platform": "android", "captureFile": "<screen>.android.xml", "commit": "<that platform's capture commit>" },
      "timestamp": "<iso8601>",
      "url": "<screen>@android",
      "violations": [
        {
          "id": "color-contrast",
          "impact": "serious",
          "tags": ["cat.color", "wcag2aa", "wcag143", "wcag21aa"],
          "description": "Elements must meet minimum colour contrast ratio thresholds",
          "help": "Text must have a contrast ratio of at least 4.5:1 (3:1 for large text)",
          "helpUrl": "https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html",
          "nodes": [
            {
              "target": ["~pin_header_title"],
              "html": "<page-source element snippet>",
              "impact": "serious",
              "failureSummary": "Fix any of the following:\n  Element has insufficient colour contrast of 2.9:1 (foreground #6E6E6E, background #FFFFFF, expected 4.5:1)",
              "any": [
                { "id": "color-contrast", "data": { "fgColor": "#6E6E6E", "bgColor": "#FFFFFF", "contrastRatio": 2.9, "expectedContrastRatio": "4.5:1", "fontSize": "normal" }, "message": "Element has insufficient colour contrast" }
              ],
              "all": [],
              "none": []
            }
          ]
        }
      ],
      "incomplete": [ { "id": "...", "impact": "...", "tags": ["..."], "description": "...", "help": "...", "helpUrl": "...", "nodes": [ { "target": ["..."], "html": "...", "failureSummary": "Needs manual verification: ...", "any": [], "all": [], "none": [] } ] } ],
      "passes": [],
      "inapplicable": []
    }
  ],
  "suppressed": [
    { "screen": "<screen>@android", "id": "image-alt", "target": ["..."], "matched_rule": 1, "reason": "..." }
  ],
  "summary": {
    "screens": 0,
    "violations": 0,
    "incomplete": 0,
    "suppressed": 0,
    "by_impact": { "critical": 0, "serious": 0, "moderate": 0, "minor": 0 },
    "by_rule": { "color-contrast": 0 }
  }
}
```

Notes on the axe mapping for native captures:
- `nodes[].html` holds the page-source element snippet (there is no DOM HTML).
- `nodes[].target` holds a native locator (Android `resource-id`/`content-desc`
  or xpath, iOS `name`/type+index), the analog of a CSS selector.
- Each finding's check goes in `nodes[].any[]` with `id` = rule id, `data` =
  the machine-readable detail (contrast colours/ratio, etc.), and `message`.
  `all`/`none` stay empty. This matches axe's node result shape closely enough
  for axe report tooling to ingest.
- `capture_sources` records where the captures came from, and
  `testEnvironment.commit` repeats that platform's commit on each result so a
  single result object is self-describing. Keep Android and iOS separate: the
  two archives are refreshed independently, so one commit for the whole report
  would be wrong. Use `"unknown"` rather than falling back to `scan_commit`.
- `passes`/`inapplicable` are left empty — a static capture cannot assert a
  criterion passed, so unflagged elements are simply not reported.

**`A11Y-REPORT.md`** — a summary line, a per-screen section, and one row per
finding:
- Header: `{S} screens → {V} violations ({critical}/{serious}/{moderate}/{minor}), {I} incomplete, {P} suppressed`.
- Provenance line under the header, one entry per platform present, so a reader
  can tell which build was actually reviewed:
  `Captures: android @ {sha:0:8} · ios @ {sha:0:8} — scanned at
  {scan_commit:0:8}`.
  Mark a platform `unknown` when its commit could not be read. If a capture
  commit differs from `scan_commit`, say so plainly rather than hiding it —
  the captures are the thing under review, not the scanning commit.
- Per screen `## <screen>@<platform>`: a table `id | impact | rule | target | issue`
  for violations, then an `Incomplete (needs manual check)` sub-table, then the
  screenshot path.
- A final `## Suppressed` table: `screen | rule | target | matched rule | reason`.

## Step 6 — Hand back

Report to the user:

1. Counts: `{V}` violations across `{S}` screens (impact split), `{I}`
   incomplete, `{P}` suppressed. Name any unpaired/skipped captures.
2. The top few screens by violation count, one line each.
3. Which commit each platform's captures came from, and — if either differs
   from `scan_commit` — that the findings describe those builds, not the
   current checkout.
4. Reminder: findings from a static capture are candidates — colour/contrast
   is estimated from the screenshot, and live-only criteria were not evaluated;
   confirm the `incomplete` items with a human or a device check.

## Constraints

- **Never execute or drive the app.** Only read the capture files.
- **Stay inside `<captures-dir>`** for captures and the component root for the
  report and fp-rules. Do not follow `..` out of the captures directory.
- **No fabricated evidence.** Every `target`/`element` must come from the page
  source you actually Read; every contrast ratio must be derived from the
  screenshot pixels, not guessed.
- **Report aggressively.** Favour recall: raise a violation whenever a rule's
  condition is met, even if you doubt it matters. Reserve `incomplete` for
  evidence you genuinely cannot read (unsamplable contrast, under-described
  nodes), and never silently drop a finding.
