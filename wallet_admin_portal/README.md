# Wallet Admin Portal

This "Wallet Admin Portal" is a Vue 3 + Vite web GUI for administrative tasks.
It provides access to support functions such as (de)blocking and revocation,
by calling an administrative API on the wallet backend.

## Development

```sh
pnpm install        # install dependencies
pnpm build          # type-check and build for production
pnpm type-check     # run vue-tsc type checking
pnpm test:unit      # run unit tests (vitest, watch mode)
pnpm coverage       # run unit tests with coverage
pnpm format         # format src/ with oxfmt
pnpm lint           # run oxlint + eslint with auto-fix
pnpm dev            # start dev server with hot-reload
pnpm preview        # preview the production build locally
```

## Deployment

Deployment can be done in a variety of ways. In our case, we deploy using Helm.
Consequently, [we have Helm charts available](../deploy/helm-charts/admin-portal).
