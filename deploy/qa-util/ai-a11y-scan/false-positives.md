# Accepted false positives and known issues

This file lists accessibility findings that have been reviewed and should **not
be raised again** on every run, in two kinds:

- **False positives** (`## False positives`) — genuine false positives
  (something a static capture misreads) or documented, accepted design
  decisions. These are permanent.
- **Known issues** (`## Known issues`) — real true-positive findings that are
  already **tracked by a ticket** and will be fixed there. These are suppressed
  only so the backlog isn't re-reported every run. **Remove the entry once its
  ticket is delivered and verified**, so the scan resumes catching regressions
  on that screen.

`/a11y-scan` applies these rules during **suppression** (Step 4): a violation
that clearly matches a rule below is moved out of `violations` into the report's
`suppressed[]` list instead of being raised again every run.

## How to add an entry

Add a new `### <short title>` section under the correct heading. Write the
rule as prose the scan can pattern-match a finding against: name the **screen**
(as it appears in the capture file name, e.g. `IntroductionScreen`), the
**component/element**, and the **rule class** it covers (`image-alt`,
`color-contrast`, …). Scope it as narrowly as the accepted risk actually is — a
specific element on a specific screen (or an enumerated set of screens), not a
whole rule across the app. State **why** it is accepted; for a known issue, cite
the **ticket** that tracks the fix.

Match on screen + rule class + element. Keep each rule self-contained:
the suppression step sees only the text below, so everything it needs to make
the call must be in the rule.

Suppression happens only in `/a11y-scan`; a too-broad rule can hide a genuine
regression, so scope additions tightly and review them like any other
accessibility exception. Suppression never touches `incomplete` findings — those
always surface for manual confirmation.

---

## False positives

### 1. Introduction screens — decorative illustration (image-alt)

The onboarding illustrations on the introduction/app-tour screens
(`IntroductionScreen`, `IntroductionPrivacyScreen`, `AppTourScreen`) are
purely decorative: each screen's heading and body text already convey the full
meaning, and the illustration adds no information a screen-reader user needs.
The illustrations are intentionally not given an accessibility label so they
are skipped by assistive technology rather than announced as noise.

`image-alt` (WCAG 1.1.1) findings that flag these decorative illustrations for
a missing accessible name are FALSE_POSITIVE. This rule is scoped to the
decorative illustration element on those introduction screens only; it does
**not** apply to icons, buttons, or any image that conveys information (e.g.
status icons, card artwork, QR codes), which must still be labelled.

### 2. Decorative card artwork, DigiD & STOP-sign illustrations (image-alt)

Decorative imagery that duplicates adjacent text is intentionally unlabelled and
skipped by assistive technology. `image-alt` (WCAG 1.1.1) findings are
FALSE_POSITIVE for these elements:

- `CardIssuanceInsuranceWithConsent` — the card artwork exposed as a raw
  concatenated name (e.g. "Kaart / Verzekering / Basic"); the card's title and
  details are already read from the surrounding nodes.
- `PersonalizeInform` (Android) — the central DigiD illustration with empty
  `content-desc`.
- `WalletTransferStoppedBySource-source`, `WalletTransferStoppedBySource-target`,
  `WalletTransferStoppedByTarget-source`, `WalletTransferStoppedByTarget-target`
  (Android) — the decorative STOP-sign `ImageView` with empty `content-desc`;
  the "Gestopt" heading and body convey the state.

Does **not** cover QR codes, which must carry an image role and descriptive name
(see Known issues).

### 3. Secondary grey/indigo & coloured-card text — estimated contrast (color-contrast)

Contrast for these findings is estimated from screenshot pixels. Several
"grey/indigo" captions are near-black antialiasing artefacts (measured ~13:1 on
device), and coloured-card text uses the approved brand palette. `color-contrast`
(WCAG 1.4.3) findings are FALSE_POSITIVE for the secondary text / coloured-card
text on these screens (verify on a real device if in doubt):

- `About` — version/config footer metadata.
- `CardData` — caption labels (e.g. "Huisnummer").
- `CardDataRevoked` — magenta revoked-message body text.
- `CardDetail` — "Geldig tot …" and list-row subtitles.
- `CardIssuanceInsuranceWithConsent` — "Basic" subtitle on the mint card.
- `ChoosePinErrorSequentialDigits` — keypad digits behind the dialog.
- `ChoosePinErrorTooFewUniqueDigits` — body text.
- `ClearDataDialog` — "JA, VERWIJDEREN" destructive button label.
- `CloseProximityQr` — "Centreer de QR-code …" instruction.
- `ConfirmPinErrorFatalMismatch` — body text.
- `DashboardWithDiplomaCard`, `DashboardWithInsuranceCard`,
  `DashboardWithLoyaltyCard` — card subtitles, card numbers, footer, and
  "Bekijk" text on the coloured cards.
- `DisclosureLoginApprove` — "Om in te loggen …" body.
- `DisclosureShareRequest` — "Alleen de volgende … gegevens" caption.
- `HistoryOverview` — activity date text.
- `OrganizationDetail` — grey caption labels.
- `PersonalizePidPreviewRenew` — subtitle and attribute captions.
- `QRScanner` — "Richt je camera op een QR-code" instruction.
- `RevocationCodeSettings` — "Je 18-tekens verwijder-code is:" intro.
- `RevocationCodeSetup` — body text.
- `WalletTransferStoppedBySource-source`,
  `WalletTransferStoppedByTarget-source` — indigo body text.
- `WalletTransferStoppedByTarget-target` — dark-mode "Sluiten" button label.

### 4. PIN dot outlines, inactive page dots & dimmed keypad (non-text-contrast)

`non-text-contrast` (WCAG 1.4.11) findings are FALSE_POSITIVE for these
decorative / inactive / dimmed graphical elements — they are not meaningful UI
component boundaries or active controls:

- `ChangePinEnterCurrent`, `ConfirmRecoverPin`, `PinUnlock` — the PIN dot
  outlines.
- `ConfirmPinErrorFatalMismatch` — the keypad dimmed behind the modal dialog.
- `IntroductionPage2` — the inactive page-indicator dots.
- `WalletTransferStoppedBySource-target`,
  `WalletTransferStoppedByTarget-target` (Android) — the "Hulp nodig?" help icon
  on the dark transfer screen.

### 5. PIN entry dots not exposed as a field (label / control-role-state)

On the iOS PIN screens the 6 PIN dots are not exposed as an accessible field;
entry progress is instead conveyed by the "Toegangscode, N cijfers over"
`StaticText`, which is the accepted approach . `label` / `control-role-state` (WCAG 4.1.2)
findings on the PIN dots are FALSE_POSITIVE for:
`ChangePinConfirmNew`, `ChangePinEnterCurrent`, `ChangePinSelectNew`,
`ConfirmRecoverPin`, `PersonalizeConfirmPin`, `PinUnlock`, `SetupChoosePin`.

### 6. Elements occluded by a modal alert (label / button-name / screen-title)

When a modal alert/dialog overlays the screen, the underlying keypad, delete
key and screen title are absent from the accessibility tree by design (the
modal traps focus). Findings that flag them as missing name/title are
FALSE_POSITIVE:

- `ChoosePinErrorSequentialDigits` — keypad digits (`button-name`) and
  "Kies een 6-cijferige pincode" title (`screen-title`).
- `PinErrorIncorrect` — keypad digits (`label`), delete key (`button-name`) and
  "Open met je pincode" title (`screen-title`).

### 7. Flattened cards & attribute label/value rows (structure)

Attribute label+value pairs are intentionally grouped into a single node so a
screen reader announces "label, value" as one unit, and card content is
intentionally flattened. Card artwork regions are decorative. `structure`
(WCAG 1.3.1) findings are FALSE_POSITIVE for these as-designed groupings:
`CardData`, `CardDataRevoked`, `CardDetail` (decorative card graphic),
`CardIssuanceInsuranceWithConsent`, `DisclosureShareRequest` (data card),
`DisclosureSharedDataDetails`, `HistoryDetailIssuance`, `HistoryOverview`
(one accessible/clickable node per row), `OrganizationDetail` (info rows),
`PersonalizePidPreview`, `PersonalizePidPreviewRenew`, `RevocationCodeSetup`
(18-char code split per character), and the flattened card nodes on
`DashboardWithInsuranceCard`, `DashboardWithLoyaltyCard`,
`DashboardWithMuseumMaandkaartCard`.

### 8. Dashboards & menus without a single page title (screen-title / structure)

The dashboard and card-overview screens intentionally have no single page-level
title / `Header` landmark (the nav menu label is separate). `screen-title` and
whole-screen `structure` findings are FALSE_POSITIVE for:
`Dashboard` (Android & iOS — including the "Activiteiten vandaag" /
"Persoonsgegevens" section headings), `DashboardCards`,
`DashboardWithLoyaltyCard`. Also `CardData` (iOS) `screen-title` where the
title is merely truncated because the view is scrolled.

Does **not** cover other screens.

### 9. Small inner label where the row/container is the tap target (target-size)

The "Bekijk Persoonsgegevens" control reports a small (74×24 / 74×4 pt) inner
label, but the surrounding row/container is the actual hit target and meets the
minimum. `target-size` (WCAG 2.5.8) findings on this inner label are
FALSE_POSITIVE for: `DisclosureApproveOrganization`,
`DisclosureBasedIssuanceDetails`, `DisclosureShareRequest`, `HistoryDetailLogin`.

### 10. Non-interactive step/progress indicators flagged for size (target-size)

The "Stap N van 3" step/progress indicators are non-interactive presentation
elements, not touch targets. `target-size` (WCAG 2.5.8) findings are
FALSE_POSITIVE for: `DisclosureApproveOrganization` (Android, the 1080×15 px
progress node) and `IntroductionPage1` (the 41×6 pt step indicator).

### 11. Clipped or borderline "Hulp nodig?" & scrolled controls (target-size)

Static captures under-measure elements that are partially scrolled off-screen or
sit right at the size threshold. `target-size` (WCAG 2.5.8) findings are
FALSE_POSITIVE for: `HelpAndInfo` ("Beveiliging en toegang", clipped by the
scroll view), `PersonalizeInform` (Android, "Hulp nodig?" ~45 dp), and
`WalletTransferStoppedByTarget-target` (Android, "Hulp nodig?" ~48 dp
borderline).

### 12. Intentionally stacked or abutting action buttons (touch-target-spacing)

Paired action buttons are intentionally stacked/adjacent by design.
`touch-target-spacing` findings are FALSE_POSITIVE for:
`AttributesMissingError` (Android, "Hulp nodig?" / "Sluiten" top bar),
`InactivityLockWarning` ("UITLOGGEN" / "JA DOORGAAN"),
`ScanWithWalletDialog` ("SLUITEN" / "SCAN QR-CODE").

### 13. Static reading-order artefacts (reading-order)

These `reading-order` findings reflect only the static tree order and do not
affect the visible/interactive experience: `CloseProximityQr` (Android, the
full-screen "Sluiten" scrim duplicating the close button),
`PersonalizeConfirmPin` (remaining-count text position), `SetupChoosePin`
(iOS, status text vs. dot position). FALSE_POSITIVE.

### 14. App-tour video scrubber (control-role-state)

On `AppTourVideoPlayer` the video scrubber/slider thumb is accessible and
exposes its value; the `control-role-state` finding claiming it has no
accessible name is FALSE_POSITIVE.

### 15. Shared "Terug" back buttons & placeholder code container (label-in-name)

`label-in-name` (WCAG 2.5.3) findings are FALSE_POSITIVE for:
`BiometricsSetup` — the top arrow and bottom button legitimately share the
name "Terug"; and `RevocationCodeSetup` — the container name is placeholder
dashes ("-\n-\n-\n-") while the real code is exposed per character.

### 16. Informational text mis-flagged as a link (link-name)

On `WalletSolutionBlocked` the "Wil je meer weten? Lees meer informatie."
`StaticText` is informational copy, not an inline link, so the `link-name`
finding is FALSE_POSITIVE.

### 17. Error-message best-practice heuristics (error-suggestion)

`error-suggestion` is a best-practice heuristic; the messaging here is as
designed or no further recovery action exists. FALSE_POSITIVE for:
`ConfirmPinErrorFatalMismatch` (Android), `PinErrorIncorrect`,
`WalletSolutionBlocked`.

---

## Known issues

Real true-positive findings tracked by a ticket. Suppressed only to keep the
backlog from being re-reported each run. **Delete the entry once the ticket is
delivered and verified.** Replace `PVW-XXXX` with the real ticket number.

### Missing heading semantics on titles & dialog titles — PVW-6332

Screen and dialog/alert titles exposed as plain `StaticText` / `ScrollView` /
`View` with no heading semantics (iOS `Header` trait / Android `heading="true"`
missing). `screen-title` / `structure` For: `CameraPermissionHint`, `ChangePinEnterCurrent`,
`ChoosePinErrorSequentialDigits`, `ChoosePinErrorTooFewUniqueDigits`,
`ClearDataDialog`, `ConfirmPinErrorFatalMismatch`, `ConfirmPinErrorMismatch`,
`Contact`, `FinishWalletDialog`, `InactivityLockWarning`, `PersonalizeConfirmPin`,
`PinErrorIncorrect`, `PinUnlock`, `PrivacyPolicy`, `QRScanner`, and the
`OrganizationDetail` nav-bar title. Applies to the title/dialog-title element on
those screens only.

### Unnamed disclosure "Bekijk" & URL controls (button-name / link-name) — PVW-6333

The co-located `accessible="false"` unnamed "Bekijk" `Button`, and URL-opening
controls exposed with the wrong role / raw URL as name. For:
`DisclosureApproveOrganization`, `DisclosureBasedIssuanceDetails`,
`DisclosureShareRequest`, `HistoryDetailIssuance` (unnamed "Bekijk" `Button`);
`About` (Android — the URL paragraph exposed with a button role); and
`OrganizationDetail` (website/privacy controls naming the raw URL). Applies to
those controls on those screens only.

### Wrong role/state on custom-tappable controls (control-role-state / target-size) — PVW-6334

Interactive elements exposing the wrong role/state. For: `Dashboard` (Android —
the action button reporting `clickable="false"`), `InactivityLockWarning` (the
dismiss overlay exposed as `StaticText` instead of a button), `Notifications`
(the push-meldingen `Switch` with no accessible name; the overlapping row
Button/Switch spacing is part of the same rework), and `Dashboard` (iOS —
"Over de app" carrying a redundant button trait at 41 pt, keep the link role
only). Applies to those controls on those screens only.

### Revoked-card state signalled by colour only (error-identification / use-of-color) — PVW-6336

On `CardDataRevoked` the revoked state is conveyed only by a magenta colour and
an unlabelled X icon.

### Merged version/config data rows on About (structure) — PVW-6341

On `About` (iOS) the app-version and configuration rows are merged into one
accessible node and should be split into separate labelled rows.

### CardHistory back-button visible target size (target-size) — PVW-6337

On `CardHistory` (iOS) the top back button's frame meets the minimum but the
visible glyph is small; pending a UX decision.

### CardHistory activity list semantics (structure) — PVW-6338

On `CardHistory` (iOS) the activity rows are not a list and a duplicate empty
overlay `Button` sits over them; pending dev investigation.

### InvalidIssuanceUlErrorDetails "Scrim" node exposed to AT (descriptive-headings-labels) — PVW-6339

On `InvalidIssuanceUlErrorDetails` (iOS) a background scrim is exposed with the
meaningless name "Scrim"; pending investigation of the element.

### DigiD authenticating — live status not exposed (control-role-state) — PVW-6340

On `PersonalizeAuthenticatingWithDigid` (iOS) no live status/progress is exposed
for the authenticating state; pending a feasibility/design decision.
