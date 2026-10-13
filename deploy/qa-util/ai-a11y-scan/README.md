# AI Accessibility Scan

AI-assisted accessibility review of the NL Wallet mobile app, built on a Claude
Code skill. It evaluates the screens the e2e suite already visits against the
**WCAG 2.1 AA** ruleset and produces an [axe-core](https://github.com/dequelabs/axe-core)-shaped
report. It is a sibling of [`../ai-security-scan`](../ai-security-scan) and
follows the same shape: a skill that never drives the app, a false-positive
suppression file, and gitignored reports that are review input rather than
committed content.

## Contents

| Skill | What it does | Output |
|---|---|---|
| `/a11y-scan` | Reads a folder of screen captures (page source + screenshot, one pair per screen/platform) and checks each against WCAG 2.1 AA | `A11Y-REPORT.json` + `.md` |

Supporting pieces:

- `.claude/skills/a11y-scan/wcag21aa.md` — the ruleset: every check, its axe
  rule id, WCAG tags, impact, and how to detect it from Android/iOS page source
  and the screenshot. This is where you tune or extend the checks.
- `.claude/settings.json` — the tool allowlist governing every run. It permits
  reading/searching the captures (Read/Glob/Grep/Task plus a set of search Bash
  commands), writing the report (Write/Edit), and `python3`/`sips` for sampling
  exact screenshot pixels during contrast checks. `defaultMode: default` means
  nothing off that list is auto-approved — but read it as scoping, not
  containment: `python3` is a general-purpose interpreter, so filesystem and
  network access are reachable whether or not `curl` is listed. That is
  accepted deliberately; see
  [docs/a11y.md](docs/a11y.md#safety-and-trust-model) for the trust model it
  rests on. (`.claude/settings.local.json` may hold personal, untracked
  extras.)
- `false-positives.md` — reviewed-and-accepted findings the scan suppresses
  instead of re-raising (see below).
- `validate-report.sh` — completeness gate for `A11Y-REPORT.json`. Checks it
  parses, carries the wrapper keys downstream tooling indexes on, holds one
  result object per capture, and has a `summary` that agrees with the findings;
  it also prints each platform's capture commit for the log, which is reported
  but never enforced. CI runs it after the scan; run it yourself after a local
  scan (`./validate-report.sh A11Y-REPORT.json captures`) to catch a run that
  died part-way instead of trusting the file's existence.
- `docs/a11y.md` — what the scan can and cannot see, and why it emits axe form.

The skill reads the capture files and writes its report. It never builds,
launches, or drives the app. The allowlist above is the only thing bounding it,
locally and in CI alike (see [CI](#ci)).

## Collecting captures

Captures come from the e2e suite via `MobileActions.captureA11ySnapshot(screenName)`
(`uiautomation/src/main/kotlin/util/MobileActions.kt`). Call it from a test at
each screen you want reviewed:

```kotlin
captureA11ySnapshot("PinScreen")
```

Capturing is **gated on `ENABLE_A11Y_CAPTURES`**: the call is a no-op unless it
is `true`, so it can be left in the tests and only produces artefacts on capture
runs. When enabled it writes two files per call into the capture directory
(`A11Y_CAPTURE_DIR`, default `build/a11y-captures/`):

- `<screen>.<platform>.xml` — the Appium page source.
- `<screen>.<platform>.png` — a full-screen screenshot.

`<platform>` is `android` or `ios` (taken from the running driver) and
`<screen>` is the argument sanitised to `[A-Za-z0-9-]`.

The first call of a run also writes `<platform>.<environment>.device.json`
once — **per environment**, not per platform:

```json
{ "platform": "android", "environment": "browserstack", "geometry_unit": "px",
  "scale": 3.5, "density_dpi": 560, "screen_width": 1264, "screen_height": 2780,
  "device_model": "OnePlus 12R", "platform_version": "14" }
```

Two per platform, one per test run: the BrowserStack run and the run on the
macOS runner's own devices both feed captures into the same
`e2e-<platform>.zip`, and those devices differ in density (the BrowserStack
OnePlus 12R reports 560 dpi, scale 3.5). `environment` comes from
`test.config.remote`. If a platform's two files disagree on `scale`, the scan
reports target size as `incomplete` instead of picking one.

`device_model` and `platform_version` come from the live session, not
`test.config.*`: `test.config.device.name` defaults to `emulator-5554` and the
BrowserStack jobs never pass it.

`scale` is what makes the **target-size** check meaningful. Android `bounds`
are device pixels, so a 48 dp minimum cannot be checked against them directly
— on a 2.625× screen a compliant button measures 126 px, and comparing raw
pixels to 48 would pass everything. The scan divides by `scale` to get dp
(`densityDpi / 160`, read via Appium's `display_density` endpoint, so it works
on emulators and BrowserStack alike); iOS already reports points, so its
`scale` is 1.

If the density cannot be read the file is skipped rather than written with a
guess, and the scan reports target size as `incomplete` ("needs manual
verification") for those screens instead of inventing a ratio. The run logs
`display density unavailable` when that happens.

**Pass the settings as `-D` system properties** (from `uiautomation/`). The
build (`uiautomation/build.gradle.kts`) forwards `ENABLE_A11Y_CAPTURES` and
`A11Y_CAPTURE_DIR` from a `-D` property (or the process environment) into the
forked test JVM's environment, where `MobileActions` reads them via
`System.getenv`. The `-D` form is preferred because it is passed per-invocation
and so works even when a Gradle daemon is reused:

```bash
cd uiautomation
./gradlew test --tests suite.FullTestSuite \
    -DENABLE_A11Y_CAPTURES=true \
    -Dtest.config.platform.name="Android" ...   # other -Dtest.config.* as usual
```

Captures default to `uiautomation/build/a11y-captures/`; pass
`-DA11Y_CAPTURE_DIR=<abs path>` to override. If you instead export
`ENABLE_A11Y_CAPTURES=true` as a shell environment variable, run with
`--no-daemon` (or `./gradlew --stop` first) so it reaches the test JVM.

Run the four instrumented flows on each platform to populate the folder:

```bash
cd uiautomation
./gradlew test \
    --tests feature.security.SetupSecurityTests \
    --tests feature.issuance.GenericIssuanceTests \
    --tests feature.close_proximity.CloseProximityDisclosureTests \
    --tests feature.wallet_transfer.WalletTransferTests \
    -DENABLE_A11Y_CAPTURES=true \
    -Dtest.config.app.identifier="nl.ictu.edi.wallet.latest" \
    -Dtest.config.device.name="emulator-5554" \
    -Dtest.config.platform.name="Android" \
    -Dtest.config.platform.version="14.0" \
    -Dtest.config.remote=false \
    -Dtest.config.automation.name="UIAutomator2"
```

Re-run with the iOS `-Dtest.config.*` values (platform `iOS`, automation
`XCUITest`) into the same directory — Android files end `.android.{xml,png}` and
iOS files `.ios.{xml,png}`, so both platforms coexist. `WalletTransferTests`
needs the two-device (source/destination) config. Each capture also logs an
`a11y snapshot: wrote …` line, so grep the output to confirm the gate opened.

## Running locally

Start Claude Code in this directory so the skill is project-scoped, then point
it at the capture folder:

```bash
cd deploy/qa-util/ai-a11y-scan
claude

# Scan every capture in the folder
> /a11y-scan ../../../uiautomation/build/a11y-captures
# → writes ./A11Y-REPORT.json + ./A11Y-REPORT.md

# Just one platform, or one screen
> /a11y-scan ../../../uiautomation/build/a11y-captures --platform android
> /a11y-scan ../../../uiautomation/build/a11y-captures --screen PinScreen

# Drop the best-practice checks (target size, tap spacing) — strict WCAG 2.1 AA only
> /a11y-scan ../../../uiautomation/build/a11y-captures --no-best-practice
```

The scan **reports aggressively**: it favours recall and raises a `violation`
whenever a rule's condition is met on a visible element, even when it is
unclear the issue matters — a human (or the false-positive file) triages
afterward. It also runs the **best-practice checks by default** (target size,
tap spacing) alongside the WCAG 2.1 AA rules; pass `--no-best-practice` to drop
them and keep the report strictly WCAG 2.1 AA.

## Reading the results

- `A11Y-REPORT.json` is a wrapper holding one axe result object per screen
  under `results[]`, plus `suppressed[]` and a `summary`. Each result carries
  `violations` and `incomplete` in axe's node shape (`nodes[].html` is the
  page-source snippet, `nodes[].target` is a native locator).
- `A11Y-REPORT.md` is the human-readable view, grouped by screen and sorted by
  impact.
- **`violations`** are every finding whose rule condition was met on a visible
  element — raised aggressively, so expect some that a human will dismiss (move
  those into `false-positives.md`). **`incomplete`** are findings whose evidence
  a static capture cannot read at all — contrast on a busy/gradient background,
  or a node the page source under-describes — and need a human or a device
  check. Nothing is dropped silently.
- All output is a starting point for human review, never a substitute for it.
  Contrast is estimated from screenshot pixels, and live-only criteria (focus
  visibility, keyboard operability, reflow, orientation) are out of scope — see
  [docs/a11y.md](docs/a11y.md).

## Suppressing accepted findings

Some findings are real observations but not actionable — genuine false
positives, or accepted design decisions (e.g. a deliberately-unlabelled
decorative illustration). Record these once in
[`false-positives.md`](false-positives.md) so the scan marks them `suppressed`
instead of re-raising them every run.

Unlike the security scan (where suppression happens in a separate `/triage`
step), `/a11y-scan` applies suppression **during the scan**: a violation that
matches a rule is moved out of `violations` into the report's `suppressed[]`
list. Pass a different rules file with `--fp-rules <file>`, or `--no-suppress`
to see everything raw.

To accept a new finding, add a rule following the format documented at the top
of `false-positives.md`: scope it to the specific screen/element and rule
class, and say why it is accepted. Review additions the way you would any
accessibility exception — a too-broad rule can hide a genuine regression.
Suppression never touches `incomplete` findings.

## CI

`deploy/gitlab/ai-a11y-scan.yml` runs the scan headless, mirroring
[`../ai-security-scan`](../ai-security-scan). Trigger it from the GitLab
**Run pipeline** web UI with the variable `SCHEDULED=ai-a11y-scan`. Two jobs run:

1. `ai-a11y-scan-fetch-token` — reads the Claude Code OAuth token from the
   `nl-wallet-anthropic` k8s secret into a short-lived dotenv artifact.
2. `ai-a11y-scan` — downloads the capture zips from MinIO
   (`qt/quality-time/a11y-captures/e2e-{android,ios}.zip`, stored by the e2e
   jobs on `main` only), unzips both into `captures/`, and runs
   `claude -p "/a11y-scan captures" --append-system-prompt "$NONINTERACTIVE"`.
   `validate-report.sh` then gates on the report being *complete* — parseable,
   one result per capture, summary consistent — so a scan that stopped part-way
   fails the job instead of publishing a partial report downstream. It also
   prints the capture provenance, which is reported but never enforced.
   `A11Y-REPORT.json` + `.md` are collected as artifacts.

   **Capture provenance.** The two archives live at a fixed path and are
   refreshed independently by whichever e2e run last succeeded, so the Android
   and iOS captures can come from *different* commits, and both can be older
   than the commit this job runs on. The upload stamps each archive with a
   `Git-Commit-Sha` object attribute (`deploy/bin/store-functions.sh`), so the
   job reads it back per platform with `mc stat` into `captures/sources.json`:

   ```json
   { "scan_commit": "1faa5673", "android": { "commit": "bb048243", ... },
                                "ios":     { "commit": "2f5bee7e", ... } }
   ```

   The scan copies those into the report's `capture_sources`, repeats each
   platform's commit on its result objects as `testEnvironment.commit`, and
   prints a `Captures: android @ … · ios @ …` line in the Markdown. Without
   this the report carries only the *scan's* commit — which `store-artifact.sh`
   tags the upload with — implying the captures came from code they may
   predate. A platform whose commit cannot be read is recorded as `unknown`,
   never silently backfilled from `scan_commit`.

Two details make the headless run work without any prompt:

- The `before_script` marks the checkout **trusted** in `~/.claude.json`;
  headless Claude ignores the project's `.claude/settings.json` allowlist in an
  untrusted directory, and CI has no trust dialog. With the allowlist loaded,
  headless `claude -p` *denies* an uncovered tool call rather than prompting for
  it, so nothing can block waiting for input — a gap surfaces as a denial in the
  job log, and any resulting partial report is caught by `validate-report.sh`.
- The `NONINTERACTIVE` system-prompt append stops the agent ending a turn on a
  question, which headless would otherwise treat as the session being done.

If no captures are present in MinIO yet, the job exits successfully with a note.
