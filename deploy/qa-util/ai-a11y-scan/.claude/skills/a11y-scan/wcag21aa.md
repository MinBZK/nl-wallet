# WCAG 2.1 AA ruleset (native mobile)

The checks `/a11y-scan` applies to each screen capture. This ruleset covers
**every WCAG 2.1 success criterion at level A and AA** (50 criteria — the
authoritative list is `wcag21-AA-full.json` in this directory, the W3C spec
export). Each criterion is either an **active rule** the scan evaluates from a
static capture, a **best-practice** rule (off by default), or **out of scope**
because it cannot be judged from a single static page-source + screenshot pair
(it needs a live, interacting session, time-based media, multiple screens, or a
device setting change). The [Coverage map](#coverage-map) below lists all 50
with their status; nothing is silently dropped.

Where an [axe-core](https://github.com/dequelabs/axe-core) rule id exists for a
check, findings use it so the report reads like axe output. Each finding is
emitted with `id` (the axe-analogous rule id), `impact`
(`critical` | `serious` | `moderate` | `minor`), and `tags` — always including
`wcag2a`/`wcag2aa` as appropriate, the specific `wcagNNN` tag, and `wcag21aa`.
Rules tagged `best-practice` run **by default** and are excluded only with
`--no-best-practice`.

## Reporting bias — aggressive

This scan is tuned for **recall over precision**: it is better to raise a
borderline issue for a human to dismiss than to miss a real one. So:

- When a rule's condition is met on a visible, in-scope element, report a
  **violation** — do not soften it to `incomplete` merely because you are
  unsure it matters.
- When you cannot decide (decorative vs informative image, whether a label is
  set elsewhere, whether structure is conveyed), **report the violation** and
  say so in the description. Err toward flagging.
- Reserve `status=incomplete` for the narrow case where you genuinely cannot
  read the **evidence itself** — e.g. contrast colours that cannot be sampled
  from the screenshot (gradient/photo/overlap), or an element the page source
  does not describe enough to judge. "I'm not sure this is a problem" is a
  **violation**, not an incomplete; "I cannot measure this" is an incomplete.

## Reading a capture

A capture is a `<screen>.<platform>.xml` page source paired with a
`<screen>.<platform>.png` screenshot.

**Android (UiAutomator2 XML).** Accessible name comes from `content-desc`,
falling back to `text` for text elements. Relevant attributes: `class`,
`resource-id`, `content-desc`, `text`, `hint`, `clickable`, `focusable`,
`enabled`, `displayed`, `password`, `checkable`, `checked`, `bounds`
(`[x1,y1][x2,y2]`, device pixels). Flutter renders most controls as a single
node whose `content-desc` is the merged semantics label.

**iOS (XCUITest XML).** Accessible name comes from `name` (identifier or
label) and `label`; state from `value`. Relevant attributes: element type
(`XCUIElementTypeButton`, `XCUIElementTypeStaticText`, `XCUIElementTypeImage`,
`XCUIElementTypeTextField`, `XCUIElementTypeSecureTextField`, …), `name`,
`label`, `value`, `enabled`, `visible`, `accessible`, `x`/`y`/`width`/`height`
(points).

**Device metadata (`<platform>.<environment>.device.json`).** Written by
`MobileActions.captureA11ySnapshot` once per platform **per environment**, so a
platform normally has two: `android.browserstack.device.json` and
`android.local-device.device.json`.

```json
{ "platform": "android", "environment": "browserstack", "geometry_unit": "px",
  "scale": 3.5, "density_dpi": 560, "screen_width": 1264, "screen_height": 2780,
  "device_model": "OnePlus 12R", "platform_version": "14" }
```

`scale` converts the page source's raw geometry into density-independent units
(`dp` on Android, `pt` on iOS, where `scale` is 1): **`dp = px / scale`**. Any
rule comparing a measured size against a dp/pt threshold — `target-size` — must
apply it. Rules that compare geometry only against *other* geometry on the same
screen (overlap, reading order, a zero gap) need no conversion.

There are two files per platform because the captures come from two test runs
on different hardware. If a platform's files disagree on `scale`, the scale is
**unavailable** for that platform — report `target-size` as `incomplete` rather
than picking one.

An element is **interactive** if Android `clickable="true"`/`focusable="true"`
or the iOS type is a control (Button, Cell, TextField, Switch, Link, …). An
element is **visible** if Android `displayed="true"` and inside the screen
`bounds`, or iOS `visible="true"`. Ignore non-visible elements.

## Global exclusions (never report)

- Elements that are not visible on screen (`displayed="false"` / `visible="false"`).
- **Disabled** controls (`enabled="false"`) for the contrast rules — WCAG 1.4.3
  and 1.4.11 explicitly exempt inactive components.
- Purely **decorative** imagery that a sighted user also gains no information
  from (background gradients, spacers). Skip only when it is *clearly*
  decorative; if you cannot tell decorative from informative, report `image-alt`
  as a violation (aggressive reporting favours flagging — the decorative case
  can be suppressed later via `false-positives.md`).
- Text that is part of a **logo or brand name** — exempt from contrast (1.4.3).
- Duplicated nodes: Flutter/iOS often expose an interactive element and its
  label as separate nodes — report the missing name once, on the interactive node.
- Anything listed as **out of scope** below — note it as not evaluated, do not
  raise it as a violation or an incomplete.

---

## Coverage map

Status legend: **active** = evaluated by the rule named; **best-practice** =
also evaluated by default, excluded only with `--no-best-practice`; **out of
scope** = not judgeable from a static capture (reason given).

| SC | Level | Name | Status |
|----|-------|------|--------|
| 1.1.1 | A | Non-text Content | active → `image-alt` |
| 1.2.1 | A | Audio-only and Video-only (Prerecorded) | out of scope — time-based media |
| 1.2.2 | A | Captions (Prerecorded) | out of scope — time-based media |
| 1.2.3 | A | Audio Description or Media Alternative | out of scope — time-based media |
| 1.2.4 | AA | Captions (Live) | out of scope — time-based media |
| 1.2.5 | AA | Audio Description (Prerecorded) | out of scope — time-based media |
| 1.3.1 | A | Info and Relationships | active → `label`, `structure` |
| 1.3.2 | A | Meaningful Sequence | active → `reading-order` |
| 1.3.3 | A | Sensory Characteristics | active → `sensory-characteristics` |
| 1.3.4 | AA | Orientation | out of scope — needs device rotation |
| 1.3.5 | AA | Identify Input Purpose | active → `identify-input-purpose` |
| 1.4.1 | A | Use of Color | active → `use-of-color` |
| 1.4.2 | A | Audio Control | out of scope — auto-playing audio |
| 1.4.3 | AA | Contrast (Minimum) | active → `color-contrast` |
| 1.4.4 | AA | Resize Text | out of scope — needs dynamic text scaling |
| 1.4.5 | AA | Images of Text | active → `image-of-text` |
| 1.4.10 | AA | Reflow | out of scope — needs 320 CSS-px reflow |
| 1.4.11 | AA | Non-text Contrast | active → `non-text-contrast` |
| 1.4.12 | AA | Text Spacing | out of scope — needs text-spacing override |
| 1.4.13 | AA | Content on Hover or Focus | out of scope — needs interaction |
| 2.1.1 | A | Keyboard | out of scope — needs interaction |
| 2.1.2 | A | No Keyboard Trap | out of scope — needs interaction |
| 2.1.4 | A | Character Key Shortcuts | out of scope — needs interaction |
| 2.2.1 | A | Timing Adjustable | out of scope — needs runtime timing |
| 2.2.2 | A | Pause, Stop, Hide | out of scope — needs motion over time |
| 2.3.1 | A | Three Flashes or Below Threshold | out of scope — needs video |
| 2.4.1 | A | Bypass Blocks | out of scope — cross-page web concept |
| 2.4.2 | A | Page Titled | active → `screen-title` |
| 2.4.3 | A | Focus Order | out of scope — needs focus traversal |
| 2.4.4 | A | Link Purpose (In Context) | active → `link-name` |
| 2.4.5 | AA | Multiple Ways | out of scope — cross-page/site concept |
| 2.4.6 | AA | Headings and Labels | active → `descriptive-headings-labels` |
| 2.4.7 | AA | Focus Visible | out of scope — needs focus interaction |
| 2.5.1 | A | Pointer Gestures | out of scope — needs interaction |
| 2.5.2 | A | Pointer Cancellation | out of scope — needs interaction |
| 2.5.3 | A | Label in Name | active → `label-in-name` |
| 2.5.4 | A | Motion Actuation | out of scope — needs device motion |
| 3.1.1 | A | Language of Page | out of scope — app-level locale, not in capture |
| 3.1.2 | AA | Language of Parts | out of scope — needs lang markup |
| 3.2.1 | A | On Focus | out of scope — needs interaction |
| 3.2.2 | A | On Input | out of scope — needs interaction |
| 3.2.3 | AA | Consistent Navigation | out of scope — needs multiple screens |
| 3.2.4 | AA | Consistent Identification | out of scope — needs multiple screens |
| 3.3.1 | A | Error Identification | active → `error-identification` (when error visible) |
| 3.3.2 | A | Labels or Instructions | active → `label` |
| 3.3.3 | AA | Error Suggestion | active → `error-suggestion` (when error visible) |
| 3.3.4 | AA | Error Prevention (Legal, Financial, Data) | out of scope — needs flow context |
| 4.1.1 | A | Parsing | out of scope — HTML-only (removed in WCAG 2.2) |
| 4.1.2 | A | Name, Role, Value | active → `button-name`, `control-role-state` |
| 4.1.3 | AA | Status Messages | out of scope — needs runtime live-region behaviour |

---

## Active rules

Grouped by WCAG principle. Apply all of these on every run.

### Perceivable

#### image-alt — 1.1.1 Non-text Content (A)

- **tags:** `cat.text-alternatives`, `wcag2a`, `wcag111`, `wcag21aa`
- **impact:** critical
- **detect:** an informative image/icon with no accessible name.
  - Android: `class` contains `ImageView`/`ImageButton` (or a Flutter node
    representing an image) with empty `content-desc` and empty `text`.
  - iOS: `XCUIElementTypeImage` with empty `name` and empty `label`.
- **skip:** decorative images (see global exclusions); images whose adjacent
  visible text already conveys the same information.

#### label — 1.3.1 Info and Relationships (A) / 3.3.2 Labels or Instructions (A) / 4.1.2 (A)

- **tags:** `cat.forms`, `wcag2a`, `wcag131`, `wcag332`, `wcag412`, `wcag21aa`
- **impact:** critical
- **detect:** a text-entry field with no programmatic label, or a control
  requiring input with no label/instructions.
  - Android: `EditText` (or Flutter text field) with empty `content-desc`,
    empty `text`, and empty `hint`.
  - iOS: `XCUIElementTypeTextField`/`XCUIElementTypeSecureTextField` with empty
    `name` and `label`.
- **note:** a placeholder exposed via `hint`/`value` counts as a label; a field
  with neither is a violation. Where input is required but no instruction/label
  is present at all, that is the 3.3.2 aspect of this rule.

#### structure — 1.3.1 Info and Relationships (A) [semantics]

- **tags:** `cat.semantics`, `wcag2a`, `wcag131`, `wcag21aa`
- **impact:** moderate
- **detect:** information conveyed only by visual grouping/structure with no
  programmatic equivalent — e.g. a visual heading exposed as plain text with no
  heading trait; related controls (a labelled group, a list) exposed as a flat
  run of nodes with no grouping; a table/row relationship lost in the tree.
  Judge the screenshot's visual structure against the page-source tree.
- **note:** native heading/group semantics are frequently unset — report the
  missing structure as a violation rather than assuming it is intentional.

#### reading-order — 1.3.2 Meaningful Sequence (A)

- **tags:** `cat.semantics`, `wcag2a`, `wcag132`, `wcag21aa`
- **impact:** moderate
- **detect:** the order of nodes in the page source (the order assistive tech
  will announce) does not match the meaningful visual reading order in the
  screenshot — e.g. a heading that visually precedes its body appears after it
  in the tree, or columns are interleaved. Report a violation when the tree
  order would mislead a screen-reader user; only use `incomplete` if the tree
  is too sparse to determine order at all.

#### sensory-characteristics — 1.3.3 Sensory Characteristics (A)

- **tags:** `cat.semantics`, `wcag2a`, `wcag133`, `wcag21aa`
- **impact:** moderate
- **detect:** visible instruction text that identifies a control **only** by
  shape, size, colour, or position — "tap the green button", "use the button on
  the right", "the round icon below". Read the instruction text from the page
  source/screenshot; report when no non-sensory identifier (a name/label) is
  also given.

#### identify-input-purpose — 1.3.5 Identify Input Purpose (AA)

- **tags:** `cat.forms`, `wcag2aa`, `wcag135`, `wcag21aa`
- **impact:** serious
- **detect:** an input that collects information **about the user** (name,
  email, phone, address, one-time code, …) that does not declare an autofill
  purpose — Android `autofillHints`, iOS `textContentType`. Report a
  **violation** when the field's purpose is evident from its label/hint/context
  but no autofill purpose is exposed in the page source. Only fall back to
  `incomplete` if the page source genuinely cannot show whether the hint is set
  (i.e. you cannot read the evidence), not merely because it "rarely exposes"
  it — err toward flagging.

#### use-of-color — 1.4.1 Use of Color (A)

- **tags:** `cat.color`, `wcag2a`, `wcag141`, `wcag21aa`
- **impact:** serious
- **detect (from the screenshot):** information conveyed by colour alone with
  no secondary cue — an error/required field distinguished only by red text
  with no icon/label/text change; a selected tab or link shown only by colour.
  Cross-check the page source for a textual/state equivalent
  (`content-desc`/`value`) before reporting.

#### color-contrast — 1.4.3 Contrast (Minimum) (AA)

- **tags:** `cat.color`, `wcag2aa`, `wcag143`, `wcag21aa`
- **impact:** serious
- **detect (from the screenshot):** visible text whose contrast against its
  background is below **4.5:1**, or **3:1** for large text (≥ 24px / ≥ 18.66px
  bold — judge from the rendered size). Locate the text node in the page source
  for the `target`/`html`, and read the actual colours from the pixels around
  it in the screenshot.
- **report `data`:** estimated foreground/background hex colours, computed
  ratio, required ratio, and font size class (`normal`/`large`).
- **incomplete, not violation, when:** the text sits on a gradient, photograph,
  or busy background where a single ratio is not meaningful, or the colours
  cannot be read confidently from the screenshot.
- **skip:** disabled controls; logotype/brand text; purely decorative text.

#### image-of-text — 1.4.5 Images of Text (AA)

- **tags:** `cat.text-alternatives`, `wcag2aa`, `wcag145`, `wcag21aa`
- **impact:** moderate
- **detect:** an image element (per the page source) that visually renders
  readable, information-bearing text in the screenshot, where that text is not
  a logo and could have been real text. Report so it can be replaced with live
  text (which also scales and re-colours).

#### non-text-contrast — 1.4.11 Non-text Contrast (AA)

- **tags:** `cat.color`, `wcag2aa`, `wcag1411`, `wcag21aa`
- **impact:** serious
- **detect (from the screenshot):** a meaningful UI component boundary or state
  indicator — button outline, input border, focus ring, toggle, meaningful icon
  — with contrast below **3:1** against adjacent colours.
- **skip:** components with a visible text label that already carries the
  meaning; disabled components; purely decorative graphics.

### Operable

#### screen-title — 2.4.2 Page Titled (A)

- **tags:** `cat.semantics`, `wcag2a`, `wcag242`, `wcag21aa`
- **impact:** moderate
- **detect:** the screen has no discernible title/heading identifying its topic
  or purpose — no top-of-screen heading text and no screen-level accessibility
  title. Report a violation when no title-like element is present; a screen
  whose purpose is unmistakably carried by a single prominent heading label is
  fine.

#### link-name — 2.4.4 Link Purpose (In Context) (A) / 4.1.2 (A)

- **tags:** `cat.name-role-value`, `wcag2a`, `wcag244`, `wcag412`, `wcag21aa`
- **impact:** serious
- **detect:** a link/hyperlink-style control whose accessible name is empty or
  non-descriptive out of context (`"here"`, `"read more"`, `"link"`). Fold plain
  unnamed tappable controls into `button-name`; use `link-name` when the control
  opens a URL or navigates.

#### descriptive-headings-labels — 2.4.6 Headings and Labels (AA)

- **tags:** `cat.semantics`, `wcag2aa`, `wcag246`, `wcag21aa`
- **impact:** moderate
- **detect:** a heading or a control/field label that is present but not
  descriptive of its topic or purpose (empty, a placeholder like `"Label"`,
  a bare index, or otherwise uninformative). Distinct from `label`/`button-name`
  (which fire when a name is **absent**); this fires when a name **exists but is
  unhelpful**.

#### label-in-name — 2.5.3 Label in Name (A)

- **tags:** `cat.name-role-value`, `wcag2a`, `wcag253`, `wcag21aa`
- **impact:** serious
- **detect:** a control's visible text label (from the screenshot / `text`) is
  not contained in its accessible name (`content-desc` / `name`). This breaks
  voice-control users who say the visible label. Report when the accessible
  name omits or contradicts the visible text (allowing for reasonable
  punctuation/case differences).

### Understandable

#### error-identification — 3.3.1 Error Identification (A)

- **tags:** `cat.forms`, `wcag2a`, `wcag331`, `wcag21aa`
- **impact:** serious
- **detect (when an error state is visible):** a field shown in an error state
  (red outline, error styling in the screenshot) whose error is not identified
  in text — no error message and no accessible description saying what is wrong.
  Only applies to screens that actually show an error; otherwise not applicable.

#### error-suggestion — 3.3.3 Error Suggestion (AA)

- **tags:** `cat.forms`, `wcag2aa`, `wcag333`, `wcag21aa`
- **impact:** moderate
- **detect (when an error state is visible):** an identified input error where a
  correction is knowable but no suggestion is offered (e.g. "invalid date" with
  no expected format). Only applies where an error is visible and a suggestion
  would not compromise security/purpose.

### Robust

#### button-name / control-role-state — 4.1.2 Name, Role, Value (A)

- **tags:** `cat.name-role-value`, `wcag2a`, `wcag412`, `wcag21aa`
- **impact:** critical
- **detect:** an interactive control missing its name, role, or state.
  - **name** (`button-name`): Android `clickable="true"` element with empty
    `content-desc` and empty `text`; iOS Button/Switch/Cell with empty `name`
    and `label`. An icon-only button with no label is the most common native
    violation.
  - **role/state** (`control-role-state`): a control whose role is not conveyed
    (a tappable `View` that reads as static text, not a button) or whose state
    is not exposed (a toggle/checkbox/selected tab with no `checked`/`value`/
    selected state in the page source). Report the missing role or state.

---

## Best-practice rules (run by default; excluded only with `--no-best-practice`)

Common mobile-accessibility expectations that are **not** WCAG 2.1 level AA
success criteria (they are AAA, WCAG 2.2, or platform guidance). They **run by
default** alongside the AA rules — in line with the aggressive reporting bias,
we surface them so a human can triage. Pass `--no-best-practice` to drop them
and keep the report strictly WCAG 2.1 AA.

### target-size — 2.5.5 Target Size (AAA, 2.1) / 2.5.8 (AA, 2.2)

- **tags:** `cat.sensory-and-visual-cues`, `best-practice`, `wcag255`
- **impact:** serious
- **detect:** an interactive element whose hit target is smaller than
  **48×48 dp** (Android) or **44×44 pt** (iOS), and not immediately adjacent
  to an equivalent larger target.

  **Convert to density-independent units first.** `bounds` /
  `width`×`height` are raw `geometry_unit` values, so on Android they are
  device pixels and comparing them to 48 directly fails every control on a
  modern phone (at `scale` 2.625 a compliant 48 dp button measures 126 px). Use
  the `scale` from the capture's device metadata:

  ```
  dp = px / scale        # Android; scale = densityDpi / 160
  pt = pt / 1.0          # iOS already reports points
  ```

  Report the dp/pt figure in `failureSummary` and put both in `data`, e.g.
  `{"width": 40, "height": 40, "unit": "dp", "scale": 2.625, "rawWidth": 105,
  "rawHeight": 105, "minSize": 48}`.

- **no `scale` available:** if the device metadata is missing, unreadable, or
  has no usable `scale`, do **not** guess a density and do **not** compare raw
  pixels against 48. Emit a single `status=incomplete` for the screen with
  `failureSummary` starting `Needs manual verification:` and explaining that
  target size could not be measured without the display density.

### touch-target-spacing — best practice

- **tags:** `cat.sensory-and-visual-cues`, `best-practice`
- **impact:** minor
- **detect:** adjacent interactive targets with no gap between their `bounds`,
  making them easy to mis-tap. A zero gap is unit-independent, so this rule
  needs no `scale`; quote any gap you do report in dp/pt per `target-size`.

---

## Out of scope (not statically checkable)

These A/AA criteria appear in the coverage map as **out of scope**. Note them
as not evaluated if relevant; do **not** report them as violations or
incompletes. Each needs something a single static capture cannot provide:

- **Time-based media** — 1.2.1, 1.2.2, 1.2.3, 1.2.4, 1.2.5, 1.4.2: audio/video
  content and its captions/descriptions; no media in a static frame.
- **Dynamic display / adaptation** — 1.3.4 (orientation), 1.4.4 (resize text),
  1.4.10 (reflow), 1.4.12 (text spacing), 1.4.13 (content on hover/focus): need
  a device rotation, zoom, text-scaling, or hover/focus interaction.
- **Interaction / input** — 2.1.1, 2.1.2, 2.1.4 (keyboard), 2.5.1, 2.5.2, 2.5.4
  (pointer/motion), 3.2.1 (on focus), 3.2.2 (on input): need the app to be
  driven, not a snapshot.
- **Timing & motion** — 2.2.1, 2.2.2, 2.3.1: need behaviour observed over time.
- **Focus behaviour** — 2.4.3 (focus order), 2.4.7 (focus visible): need focus
  traversal on a live screen.
- **Cross-page / cross-screen** — 2.4.1 (bypass blocks), 2.4.5 (multiple ways),
  3.2.3 (consistent navigation), 3.2.4 (consistent identification): need more
  than one screen or a whole-app view.
- **Language** — 3.1.1 (language of page), 3.1.2 (language of parts): app-level
  locale / inline language markup not present in the capture.
- **Flow-dependent** — 3.3.4 (error prevention for legal/financial/data): needs
  a multi-step process with confirmation/reversal.
- **Runtime robustness** — 4.1.1 (parsing, HTML-only and removed in WCAG 2.2),
  4.1.3 (status messages): need HTML or a runtime live-region announcement.
