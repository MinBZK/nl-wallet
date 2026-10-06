# /a11y-scan

Evaluates a folder of NL Wallet screen captures against **WCAG 2.1 AA** and
writes an axe-form report.

## Files

- `SKILL.md` — the orchestrator (arguments, fan-out, suppression, output schema).
- `wcag21aa.md` — the ruleset. Covers all 50 WCAG 2.1 A/AA success criteria: a
  coverage map (each criterion marked active / best-practice / out-of-scope with
  a reason) plus detailed active rules with axe rule id, WCAG tags, impact, and
  how to detect each from Android/iOS page source and the screenshot. Edit this
  to add or tune checks.
- `wcag21-AA-full.json` — the W3C spec export the coverage map is derived from
  (the authoritative criterion list). Reference only; the scan works off
  `wcag21aa.md`.

## Input

Capture pairs written by `MobileActions.captureA11ySnapshot(screenName)` in
`uiautomation/` (only when `ENABLE_A11Y_CAPTURES=true`; otherwise the call is a
no-op):

- `<screen>.<platform>.xml` — Appium page source (UiAutomator2 / XCUITest).
- `<screen>.<platform>.png` — full-screen screenshot.

`<platform>` is `android` or `ios`; `<screen>` is the sanitised screen name.
The XML is required; the screenshot is used for every colour/contrast rule and
to distinguish informative from decorative imagery. A capture with no
screenshot is still evaluated from the page source (contrast rules become
`incomplete`).

## Output

- `A11Y-REPORT.json` — a wrapper holding one [axe-core](https://github.com/dequelabs/axe-core)
  result object per screen (`results[]`), plus `suppressed[]` and a `summary`.
  Each result has `violations` and `incomplete` in axe's node shape, with the
  page-source snippet in `nodes[].html` and a native locator in `nodes[].target`.
- `A11Y-REPORT.md` — human-readable, grouped by screen.

## Behaviour notes

- **Aggressive reporting (recall over precision).** A rule whose condition is
  met on a visible element is raised as a `violation`, even when it is unclear
  the issue matters — a human (or the suppression file) triages afterward.
  `incomplete` is reserved for evidence that cannot be read at all (contrast on
  gradient/photo backgrounds, under-described nodes); findings are never dropped
  silently.
- **Best-practice checks run by default.** Target size and tap spacing are
  tagged `best-practice` and included in every run; pass `--no-best-practice`
  to drop them and keep the report strictly WCAG 2.1 AA.
- **Suppression is in-scan.** `false-positives.md` (or `--fp-rules <file>`) is
  applied in Step 4; matched violations move to `suppressed[]`.
- **Read-only with respect to the target.** The skill reads captures and
  writes its report; it never builds, launches, or drives the app. Live-only
  criteria (focus visibility, keyboard operability, reflow, orientation) are
  out of scope by design.
