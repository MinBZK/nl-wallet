# Browser tests

[Playwright](https://playwright.dev/) tests for the NL Wallet web surfaces. This
is a pnpm workspace with one package per surface under [`packages/`](packages/):

| Package                                           | Base-URL env var        |
| ------------------------------------------------- | ----------------------- |
| [`wallet-web`](packages/wallet-web)               | `DEMO_INDEX_URL`        |
| [`revocation-portal`](packages/revocation-portal) | `REVOCATION_PORTAL_URL` |
| [`fallback-pages`](packages/fallback-pages)       | `UNIVERSAL_LINK_BASE`   |
| [`admin-portal`](packages/admin-portal)           | `ADMIN_PORTAL_URL`      |

Each `wallet-web` and `revocation-portal` test performs both an
[axe-core](https://github.com/dequelabs/axe-core) accessibility check (WCAG 2.1
A/AA) and a visual snapshot assertion. `admin-portal` runs the accessibility
check only, and `fallback-pages` does neither.

## Prerequisites

- **Node.js ≥ 22.13** (the pnpm version this repo tracks requires it). With
  [nvm](https://github.com/nvm-sh/nvm): `nvm install 22 && nvm use 22`.
- **pnpm** via Corepack (bundled with Node): `corepack enable pnpm`.
- **Playwright browsers**: installed via `npx playwright install` (see below).
  Local configs use only Chromium; CI configs also use WebKit.

## Setup

From this directory (`browsertests/`):

```bash
pnpm install --frozen-lockfile
npx playwright install chromium      # add `webkit` too if you run the CI config
```

## Running the tests

Each package exposes two scripts:

- `test:local` —
  [`config/local.config.js`](packages/wallet-web/config/local.config.js):
  Chromium desktop only.
- `test:ci` — [`config/ci.config.js`](packages/wallet-web/config/ci.config.js):
  Chromium **and** WebKit × desktop/tablet/mobile viewports, plus an Allure
  reporter. Mirrors the pipeline.

The base URL comes from an environment variable per package (see the table).

### fallback-pages (no backend)

The script already exports `` (the hosted test env), so this just needs network
access:

```bash
cd packages/fallback-pages
UNIVERSAL_LINK_BASE= "" pnpm run test:local
```

### wallet-web

```bash
cd packages/wallet-web
DEMO_INDEX_URL="" pnpm run test:local
```

### revocation-portal

```bash
cd packages/revocation-portal
REVOCATION_PORTAL_URL="" pnpm run test:local
```

### admin-portal

Needs the base URL plus a password per test user, because every test signs in
through KeyCloak:

| Env var                  | Value                                         |
| ------------------------ | --------------------------------------------- |
| `ADMIN_PORTAL_URL`       | Base URL of the portal                        |
| `AP_OPERATOR1_PASSWORD`  | Password of the `operator1` user (Beheerder)  |
| `AP_TEAMLEAD1_PASSWORD`  | Password of the `teamlead1` user (Teamleider) |
| `AP_SUPERUSER1_PASSWORD` | Password of the `superuser1` user (Superuser) |

```bash
cd packages/admin-portal
ADMIN_PORTAL_URL="http://localhost:5173" \
  AP_OPERATOR1_PASSWORD="operator1" \
  AP_TEAMLEAD1_PASSWORD="teamlead1" \
  AP_SUPERUSER1_PASSWORD="superuser1" \
  pnpm run test:local
```

A `setup` project ([`auth.setup.js`](packages/admin-portal/tests/auth.setup.js))
signs each user in once and stores the session under `playwright/.auth/`, which
the browser projects reuse through `storageState`. The tests that cover signing
in, signing out and rejecting bad credentials log in for real instead. Note that
the portal session has an absolute 15-minute expiry
(`admin_portal.session_ttl_in_ms`) that requests do not extend, so a run lasting
longer than that outlives its stored sessions.

Those three users are seeded by
[`realm.json`](../scripts/devenv/keycloak/import/realm.json), which gives each
one its own username as password. In CI the passwords come from the
`nl-wallet-keycloak-passwords` secret instead — see
[`automation-tests.yml`](../deploy/gitlab/automation-tests.yml).

Serving the portal locally takes three processes, the last two in their own
shell:

```bash
scripts/start-devenv.sh kc     # KeyCloak, from the repo root
cd wallet_admin_portal
pnpm mock:auth-server          # backend (BFF) mock on :3000
pnpm dev                       # SPA on :5173
```

What the tests assert per user — display name, role, sidebar entries and the
task actions they may create — lives in
[`data/users.js`](packages/admin-portal/data/users.js) and has to match the
privileges KeyCloak grants them.

Two axe rules (`select-name` and `scrollable-region-focusable`) are disabled in
[`AdminPortal.test.js`](packages/admin-portal/tests/AdminPortal.test.js) for
known portal defects; re-enable each one as its issue is fixed.

### Useful flags

Invoke Playwright directly for more control (keep the env var):

```bash
DEMO_INDEX_URL="http://localhost:3005/" npx playwright test --config=config/local.config.js \
  --headed          # watch the browser
  # --ui            # interactive UI mode
  # --debug         # step through with the inspector
```

## Visual snapshots on macOS

`wallet-web` and `revocation-portal` assert with `toHaveScreenshot`, and the
committed baselines are **Linux** images (`…-linux.png`, generated in the CI
Playwright container). On macOS Playwright looks for `…-darwin.png`, finds none,
and **fails those assertions** with a missing-baseline error.

To generate local (darwin) baselines, add `--update-snapshots` — but **do not
commit** the generated PNGs:

```bash
DEMO_INDEX_URL="http://localhost:3005/" pnpm run test:local -- --update-snapshots
```

## Linting & formatting

Run this from the workspace root

Run from `/browsertests`:

```bash
pnpm run lint
pnpm run format
```
