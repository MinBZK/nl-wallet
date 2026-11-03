---
name: a11y-scan
description: >-
  AI accessibility scan of NL Wallet screen captures against WCAG 2.1 AA.
  Reads a folder of Appium page-source XML + screenshot pairs (one per
  screen/platform), spawns one review subagent per screen, applies the
  false-positive suppression file, and writes a raw axe-core A11Y-REPORT.json
  (a bare array of result objects, drop-in for axe tooling) alongside an
  A11Y-REPORT.meta.json sidecar and a human-readable A11Y-REPORT.md.
  Read-only — it inspects captures, never builds or drives the app. Use
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
Produces `A11Y-REPORT.json` — a **raw [axe-core](https://github.com/dequelabs/axe-core)
report**: a bare JSON array of result objects, one per screen, byte-compatible
with axe's own output so it drops straight into axe report tooling. The scan's
own metadata that axe has no place for — capture provenance, the suppression
list, and the run summary — lives in a sibling `A11Y-REPORT.meta.json`, and a
human-readable `A11Y-REPORT.md` accompanies both.

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

OUTPUT — findings first, then exactly one <coverage> block, nothing else.

A) One <finding> block per violation and per incomplete:

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

B) Exactly one <coverage> block that accounts for EVERY active rule id on this
screen. A real axe run reports each rule as violations / incomplete / passes /
inapplicable; this block supplies the pass and inapplicable dispositions so the
collator can build those two buckets (the finding blocks above already cover
violations and incomplete). Every active rule id must appear in exactly one
place: a <finding> above, a `pass:` line, or the `inapplicable:` list — never
more than one, never omitted.

- `pass:` — the rule applies to this screen (at least one in-scope element is
  the kind the rule governs) and every such element satisfies it. Give one
  representative passing element per line. If you cannot name a concrete
  passing element, the rule is `inapplicable`, not a pass.
- `inapplicable:` — the screen has no element the rule governs (e.g. `label` on
  a screen with no input, `error-identification` with no visible error state).

<coverage>
pass:
  {rule_id} | {target locator} | {page-source element snippet, one node} | {one line: why it satisfies the rule}
  ...
inapplicable:
  {rule_id}
  ...
</coverage>

A clean screen (no violations, no incompletes) therefore emits zero <finding>
blocks and a <coverage> block whose `pass:`/`inapplicable:` lists together name
every active rule.
```

## Step 3 — Collate

1. Collect every `<finding>` and the one `<coverage>` block from each subagent.
2. **Light dedupe within a screen:** if two findings share the same `rule_id`
   and the same `target`, keep the one with the longer description.
3. Group by screen (`{screen}.{platform}`). For each screen, bucket every active
   rule into exactly one of the four axe lists, so the four together account for
   the whole active rule set on that screen:
   - **violations** — findings with `status=violation`.
   - **incomplete** — findings with `status=incomplete`.
   - **passes** — rules named on a `pass:` line (with their representative node).
   - **inapplicable** — rules named in the `inapplicable:` list.
   A rule that shows up in a finding is never also emitted as a pass or
   inapplicable. If a subagent left an active rule out of all four, treat it as
   `inapplicable` for that screen (and note the gap in the hand-back) rather than
   dropping it — the report must cover every active rule on every screen, the
   way axe does. Best-practice rules are part of the active set only when they
   are enabled; when `--no-best-practice` is given they appear in no bucket.

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

3. Move each suppressed finding out of its screen's `violations` and record it
   in the **meta sidecar's** `suppressed[]` (Step 5) with the finding, the
   matched rule number, and the reason. The axe `A11Y-REPORT.json` has no place
   for a suppression list, so a suppressed finding simply does not appear in any
   result object's `violations`; the record lives only in `A11Y-REPORT.meta.json`.
   A suppressed rule does **not** migrate to `passes`/`inapplicable` — it was a
   real match, just an accepted one.

## Step 5 — Build the report

Two JSON files plus the Markdown. `A11Y-REPORT.json` is a **verbatim axe-core
report** — a bare JSON array of result objects, one per screen, identical in
shape to what `axe.run()` serialises — so downstream axe tooling ingests it with
no translation. Everything the scan adds that axe has no slot for goes in the
`A11Y-REPORT.meta.json` sidecar.

Assign stable ids `A-001`, `A-002`, … across all surviving violations/incomplete
findings ordered by (impact desc: critical>serious>moderate>minor, then screen,
then rule_id) — these ids are used in the Markdown and the meta sidecar, not in
the axe JSON (axe has no per-finding id field).

**Write each file in its own step, as the sole substantial content of that
response.** Populating all four axe buckets for every screen makes
`A11Y-REPORT.json` large, and a single response is capped at a fixed output-token
budget (`CLAUDE_CODE_MAX_OUTPUT_TOKENS`, 32000 by default — CI raises it). So:
one Write call for `A11Y-REPORT.json`, a separate one for `A11Y-REPORT.meta.json`,
a separate one for `A11Y-REPORT.md`; never emit a file's contents as prose in
addition to Writing it, and keep the Step 6 hand-back a short summary, not a
paste of the report. If the array is so large that even its own Write would
exceed the budget, write the opening `[`, then append result objects in batches
with Edit across several responses, closing with `]` — each response stays under
the cap. The report still ends up byte-identical; only the number of write steps
changes.

### `A11Y-REPORT.json` — the axe array (written to this component's root)

A JSON **array** (not an object). One result object per screen capture, in
capture order. Match axe exactly: object keys and node keys are emitted in the
alphabetical order shown, and the engine/runner/toolOptions values are fixed
literals so the file is indistinguishable from an axe run.

```json
[
  {
    "inapplicable": [
      {
        "description": "Ensure form elements have labels",
        "help": "Form elements must have labels",
        "helpUrl": "https://dequeuniversity.com/rules/axe/4.12/label?application=axeAPI",
        "id": "label",
        "impact": null,
        "nodes": [],
        "tags": ["cat.forms", "wcag2a", "wcag131", "wcag332", "wcag412", "wcag21aa"]
      }
    ],
    "incomplete": [
      {
        "description": "Ensure the contrast between foreground and background colours meets WCAG 2 AA minimum contrast ratio thresholds",
        "help": "Elements must meet minimum colour contrast ratio thresholds",
        "helpUrl": "https://dequeuniversity.com/rules/axe/4.12/color-contrast?application=axeAPI",
        "id": "color-contrast",
        "impact": "serious",
        "nodes": [
          {
            "all": [],
            "any": [
              {
                "data": { "fgColor": null, "bgColor": null, "contrastRatio": null, "expectedContrastRatio": "4.5:1", "fontSize": "normal" },
                "id": "color-contrast",
                "impact": "serious",
                "message": "Unable to sample a reliable contrast ratio — text sits on a gradient background",
                "relatedNodes": []
              }
            ],
            "failureSummary": "Needs manual verification:\n  Contrast could not be measured from the screenshot (gradient background) — check on device",
            "html": "<page-source element snippet>",
            "impact": "serious",
            "none": [],
            "target": ["~onboarding_subtitle"]
          }
        ],
        "tags": ["cat.color", "wcag2aa", "wcag143", "wcag21aa"]
      }
    ],
    "passes": [
      {
        "description": "Ensure buttons have discernible text",
        "help": "Buttons must have discernible text",
        "helpUrl": "https://dequeuniversity.com/rules/axe/4.12/button-name?application=axeAPI",
        "id": "button-name",
        "impact": null,
        "nodes": [
          {
            "all": [],
            "any": [
              { "data": null, "id": "button-name", "impact": "critical", "message": "Element has an accessible name (\"Continue\")", "relatedNodes": [] }
            ],
            "html": "<node content-desc=\"Continue\" clickable=\"true\" .../>",
            "impact": null,
            "none": [],
            "target": ["~continue_button"]
          }
        ],
        "tags": ["cat.name-role-value", "wcag2a", "wcag412", "wcag21aa"]
      }
    ],
    "testEngine": { "name": "axe-core", "version": "4.12.1" },
    "testEnvironment": {
      "orientationAngle": 0,
      "orientationType": "portrait-primary",
      "userAgent": "NL Wallet android app on OnePlus 12R (Android 14)",
      "windowHeight": 2780,
      "windowWidth": 1264
    },
    "testRunner": { "name": "axe" },
    "timestamp": "<iso8601, ms + Z, e.g. 2026-10-06T05:41:05.529Z>",
    "toolOptions": { "reporter": "v1" },
    "url": "<screen>@android",
    "violations": [
      {
        "description": "Ensure the contrast between foreground and background colours meets WCAG 2 AA minimum contrast ratio thresholds",
        "help": "Elements must meet minimum colour contrast ratio thresholds",
        "helpUrl": "https://dequeuniversity.com/rules/axe/4.12/color-contrast?application=axeAPI",
        "id": "color-contrast",
        "impact": "serious",
        "nodes": [
          {
            "all": [],
            "any": [
              {
                "data": { "fgColor": "#6e6e6e", "bgColor": "#ffffff", "contrastRatio": 2.9, "expectedContrastRatio": "4.5:1", "fontSize": "normal" },
                "id": "color-contrast",
                "impact": "serious",
                "message": "Element has insufficient colour contrast of 2.9 (foreground #6e6e6e, background #ffffff, expected 4.5:1)",
                "relatedNodes": []
              }
            ],
            "failureSummary": "Fix any of the following:\n  Element has insufficient colour contrast of 2.9 (foreground #6e6e6e, background #ffffff, expected 4.5:1)",
            "html": "<node text=\"Enter your PIN\" .../>",
            "impact": "serious",
            "none": [],
            "target": ["~pin_header_title"]
          }
        ],
        "tags": ["cat.color", "wcag2aa", "wcag143", "wcag21aa"]
      }
    ]
  }
]
```

**Per result object** — exactly these ten keys, alphabetical:

- `inapplicable`, `incomplete`, `passes`, `violations` — the four rule buckets
  from Step 3. Each entry is a **rule object** (below). `incomplete`/`violations`
  carry the rules that fired on this screen; `passes`/`inapplicable` carry the
  rest of the active set, so the four lists together cover every active rule.
- `testEngine` — the fixed literal `{ "name": "axe-core", "version": "4.12.1" }`.
- `testEnvironment` — the five browser-env keys axe emits, filled from the
  platform's `<platform>.<environment>.device.json`:
  `orientationAngle` `0`; `orientationType` `"portrait-primary"` (`"landscape-primary"`
  only if `screen_width` > `screen_height`); `userAgent` a plain native
  descriptor `"NL Wallet <platform> app on <device_model> (<platform> <platform_version>)"`;
  `windowHeight`/`windowWidth` the device's `screen_height`/`screen_width`. If no
  device file is usable, keep all five keys with `windowHeight`/`windowWidth`
  `null` and a `userAgent` of `"NL Wallet <platform> app"`.
- `testRunner` — the fixed literal `{ "name": "axe" }`.
- `timestamp` — ISO-8601 with milliseconds and a `Z`, e.g. `2026-10-06T05:41:05.529Z`.
- `toolOptions` — the fixed literal `{ "reporter": "v1" }`.
- `url` — `<screen>@<platform>` (a native screen has no URL; this is axe's `url`
  slot repurposed as the screen identifier).

**Rule object** — seven keys, alphabetical: `description`, `help`, `helpUrl`,
`id`, `impact`, `nodes`, `tags`.

- `id`, `tags` come from the rule's entry in `wcag21aa.md`; `description` and
  `help` from its row in the ruleset's **Axe rule catalog**.
- `helpUrl` is always `https://dequeuniversity.com/rules/axe/4.12/<id>?application=axeAPI`.
- `impact` at the **rule level** is the rule's impact for `violations` and
  `incomplete`, and `null` for `passes` and `inapplicable` (axe does this).
- `nodes` is `[]` for every `inapplicable` rule; otherwise one node per element.

**Node object** — alphabetical keys `all`, `any`, (`failureSummary`), `html`,
`impact`, `none`, `target`:

- `target` — a one-element array holding the native locator (Android
  `resource-id`/`content-desc` or xpath, iOS `name`/type+index) — axe's CSS-
  selector slot.
- `html` — the page-source element snippet (there is no DOM HTML).
- `impact` — the rule's impact on `violations`/`incomplete` nodes; `null` on
  `passes` nodes.
- `any` — one **check object** `{ "data", "id", "impact", "message",
  "relatedNodes" }`: `id` = rule id, `impact` = the rule's impact,
  `message` = the human sentence, `data` = the machine detail (contrast
  colours/ratio, target-size dp, …) or `null`, `relatedNodes` = `[]`. `all` and
  `none` stay `[]`.
- `failureSummary` — present **only** on `violations` and `incomplete` nodes.
  Violations start `Fix any of the following:`; incompletes start
  `Needs manual verification:`. Omit the key entirely on `passes` nodes.

Mapping notes:
- A suppressed finding appears in **no** result object (it is recorded only in
  the meta sidecar). Nothing else is dropped silently.
- Contrast hex is lower-case (`#6e6e6e`), matching axe.
- Never fabricate a passing node: a `pass` entry must carry a real element the
  subagent named. A rule with no concrete element is `inapplicable` (empty
  `nodes`), not a pass.

### `A11Y-REPORT.meta.json` — the sidecar (same directory)

Everything the axe array has no slot for. Written whether or not there are
findings.

**Use these exact top-level key names — do not rename or restructure them.**
`validate-report.sh` gates the run on `wcag_version`, `generated_at`,
`suppressed`, and `summary` (with numeric `summary.screens` and
`summary.violations`) **by name**, and reads provenance from `capture_sources`.
Emitting a synonym instead — `schema_version` for `wcag_version`, `provenance`
for `capture_sources`, `run_options` for `rules_run` — fails the gate or
silently drops the provenance echo. `wcag_version` is the literal string
`"wcag21aa"` (the ruleset version), not a schema or format version. You may add
extra keys (e.g. a `findings` list), but never rename or omit the ones below.

```json
{
  "wcag_version": "wcag21aa",
  "generated_at": "<iso8601>",
  "captures_dir": "<captures-dir>",
  "report": "A11Y-REPORT.json",
  "capture_sources": {
    "scan_commit": "<commit of the job that ran the scan, or \"unknown\">",
    "android": { "commit": "<commit the Android captures came from>", "archive": "<minio path>" },
    "ios": { "commit": "<commit the iOS captures came from>", "archive": "<minio path>" }
  },
  "rules_run": ["image-alt", "button-name", "..."],
  "suppressed": [
    { "id": "A-007", "screen": "<screen>@android", "rule": "image-alt", "target": ["..."], "matched_rule": 1, "reason": "..." }
  ],
  "summary": {
    "screens": 0,
    "violations": 0,
    "incomplete": 0,
    "passes": 0,
    "inapplicable": 0,
    "suppressed": 0,
    "by_impact": { "critical": 0, "serious": 0, "moderate": 0, "minor": 0 },
    "by_rule": { "color-contrast": 0 }
  }
}
```

- `capture_sources` records where the captures came from (Step 1). Keep Android
  and iOS separate — the two archives are refreshed independently, so one commit
  for the whole report would be wrong — and use `"unknown"` rather than falling
  back to `scan_commit`. This used to live on each result's `testEnvironment`;
  it now lives here, because the axe result shape has no commit field.
- `summary.violations` and `summary.incomplete` must equal the counts across the
  axe array (`violations` is what survived suppression); `summary.suppressed`
  counts the `suppressed[]` entries. The gate re-counts both from the report and
  fails on a mismatch, so do not hand-tally — derive these from the buckets you
  actually wrote.

### `A11Y-REPORT.md` — the human-readable view

A summary line, a per-screen section, and one row per finding:

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
