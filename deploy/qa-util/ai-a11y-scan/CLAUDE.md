# AI Accessibility Scan

A Claude Code skill for AI-assisted accessibility review of the NL Wallet
mobile app. It is **read-only with respect to the target**: it reads the
page-source XML and screenshot captures collected by the e2e tests and writes
a report; it never builds, runs, or drives the app itself.

Available skill (`.claude/skills/`):

- `/a11y-scan` — evaluate a folder of screen captures (Appium page source +
  screenshot, one pair per screen) against the **WCAG 2.1 AA** ruleset and
  write an axe-form report → `A11Y-REPORT.json` + `.md`

The captures are produced by `MobileActions.captureA11ySnapshot(screenName)` in
`uiautomation/` (writes `<screen>.<platform>.{xml,png}`, gated on
`ENABLE_A11Y_CAPTURES` — pass `-DENABLE_A11Y_CAPTURES=true` to the Gradle test
run; `build.gradle.kts` forwards it into the test JVM). The NL Wallet repository
root is `../../..` relative to this directory. See `README.md` for usage.
