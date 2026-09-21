# Browser tests

[Playwright](https://playwright.dev/) tests for the NL Wallet web surfaces. This
is a pnpm workspace with one package per surface under [`packages/`](packages/):

| Package                                           | Base-URL env var        |
| ------------------------------------------------- | ----------------------- |
| [`wallet-web`](packages/wallet-web)               | `DEMO_INDEX_URL`        |
| [`revocation-portal`](packages/revocation-portal) | `REVOCATION_PORTAL_URL` |
| [`fallback-pages`](packages/fallback-pages)       | `UNIVERSAL_LINK_BASE`   |

Each `wallet-web` and `revocation-portal` test performs both an
[axe-core](https://github.com/dequelabs/axe-core) accessibility check (WCAG 2.1
A/AA) and a visual snapshot assertion. `fallback-pages` does neither.

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
