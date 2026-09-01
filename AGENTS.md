# NL Wallet — Agent Instructions

## Project Structure

- `wallet_core/` — Rust workspace: core business logic, servers, FFI bindings
- `wallet_app/` — Flutter mobile app (iOS & Android), uses BLoC + Clean Architecture
- `wallet_web/` — TypeScript web library (Web Components)
- `wallet_admin_portal/` — Vue 3 + Vite frontend for the wallet_provider admin endpoints
- `wallet_docs/` — Sphinx documentation source (Python)
- `scripts/` — Dev environment, code generation, migrations
- `deploy/` — Kubernetes/Helm charts

## Build & Test Commands

### Rust
```bash
cargo nextest run --manifest-path wallet_core/Cargo.toml --features integration_test
cargo clippy --manifest-path wallet_core/Cargo.toml --all-features --tests -- -Dwarnings

# format all crates (nightly rustfmt, skipping build artifacts in target/)
find wallet_core -type d -name target -prune -o -mindepth 2 -type f -name Cargo.toml -print0 \
  | xargs -0 -n1 cargo +nightly fmt --manifest-path
```

### Flutter
```bash
flutter test                                          # unit & golden tests
flutter test --tags=golden                            # golden (snapshot) tests only
flutter test --exclude-tags=golden                    # unit tests only
flutter run --dart-define=MOCK_REPOSITORIES=true      # run with mocks
dart format . --line-length 120                       # format code
flutter gen-l10n                                      # regenerate localizations
```

### Web (`wallet_web/`)
```bash
pnpm run test -- --run    # Vitest (add --run, otherwise it stays in watch mode)
pnpm run build
```

### Admin portal (`wallet_admin_portal/`, uses pnpm)
```bash
pnpm test:unit -- --run  # Vitest (add --run, otherwise it stays in watch mode)
pnpm type-check          # vue-tsc
pnpm lint                # oxlint + eslint, both with --fix
pnpm format              # oxfmt src/
pnpm build
```

### Code Generation (run after changing bridge or bindings)
```bash
scripts/generate-flutter-rust-bridge.sh   # Rust ↔ Flutter FFI
scripts/generate-web-bindings.sh          # Rust ↔ TypeScript
```

### Dev Environment
```bash
scripts/setup-devenv.sh       # first-time setup
scripts/start-devenv.sh       # start local services
scripts/migrate-db.sh         # run DB migrations
```

## Code Conventions

### Rust
- Rust edition 2024, line width 120; formatted with nightly `rustfmt` (see command above)
- Coding style: "parse, don't validate"; use the type system to exclude illegal states where feasible
  - Prefer a newtype with a private field and a fallible constructor (`TryFrom`/`fn new() -> Result<_, _>`)
    over a check the caller is expected to remember; holding the value is then proof it is valid
  - Parse untrusted input once, at the boundary (deserialization, HTTP handler, settings load); functions
    further in take already-parsed types
  - In non-test code, avoid `unwrap()`/`expect()`/`unreachable!()` on invariants established elsewhere —
    change the type so the case cannot arise (see `utils::vec_at_least` for the non-empty-collection case)
  - Push the burden of proof of invariants upward as far as is practical, but no further: don't push proof
    obligations onto callers when doing so makes the API onerous
- Imports in 3 groups (std → third-party and workspace → super/crate imports), alphabetically within each
- Each imported symbol on its own import line
- Custom error enums per module; use `thiserror`
- Async with Tokio; HTTP servers via Axum; DB via Sea-ORM
- Prefer returning iterators over heap allocated values
- In `thiserror` enum variants, use `#[source]` instead of `#[from]`
- Prefer native `async fn` in traits; use `async_trait` only when object-safety/dyn compatibility is required
  (e.g. Sea-ORM migrations, uniffi callbacks)

### Flutter/Dart
- Line length 120 (`dart format . --line-length 120`)
- BLoC pattern: `<Feature>Bloc`, `<Feature>Event`, `<Feature>State`
- State files use `freezed`; JSON via `json_serializable`
- Localization via ARB files (`flutter gen-l10n` after changes)
- Mock repositories available in `packages/wallet_mock/` for UI dev/testing

### Git
- GPG-signed commits required
- Branch naming: `PREFIX-JIRA-CODE-short-description` (e.g. `PVW-123-add-feature`)
- Commit messages: imperative mood, capitalized, no trailing period, wrap at 72 chars
- PR title follows same conventions as commit messages

## Workflow Rules

- Never auto-commit or push without being asked
- Run appropriate/relevant tests before marking any task done
- Run `cargo +nightly fmt` and `cargo clippy` after Rust changes; fix all warnings before finishing
- Run `dart format` after Dart changes
- Run `pnpm lint` and `pnpm format` after `wallet_admin_portal/` changes
- After modifying the Flutter-Rust bridge API, regenerate bindings with the script above
- Mock-OIDC integration tests run serially (configured in `.config/nextest.toml`)
