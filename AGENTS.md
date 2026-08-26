# NL Wallet — Agent Instructions

## Project Structure

- `wallet_core/` — Rust workspace (~61 crates): core business logic, servers, FFI bindings
- `wallet_app/` — Flutter mobile app (iOS & Android), uses BLoC + Clean Architecture
- `wallet_web/` — TypeScript web library (Web Components)
- `nl-rdo-max/` — DigiD connector (Python)
- `scripts/` — Dev environment, code generation, migrations
- `deploy/` — Kubernetes/Helm charts

## Build & Test Commands

### Rust
```bash
cargo nextest run --manifest-path wallet_core/Cargo.toml
cargo nextest run --manifest-path wallet_core/Cargo.toml --features integration_test
cargo clippy --manifest-path wallet_core/Cargo.toml --all-features --tests -- -Dwarnings

# format all crates (nightly rustfmt, skipping build artifacts in target/)
find wallet_core -type d -name target -prune -o -mindepth 2 -type f -name Cargo.toml -print0 \
  | xargs -0 -n1 cargo +nightly fmt --manifest-path
```

### Flutter
```bash
flutter test                                          # unit & widget tests
flutter test --tags golden                            # golden (snapshot) tests
flutter run --dart-define=MOCK_REPOSITORIES=true      # run with mocks
dart format . --line-length 120
flutter gen-l10n                                      # regenerate localizations
```

### Web
```bash
npm run test    # Vitest
npm run build
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
- Imports in 3 groups (std → third-party and workspace → super/crate imports), alphabetically within each
- Each imported symbol on its own import line
- Custom error enums per module; use `thiserror`
- Async with Tokio; HTTP servers via Axum; DB via Sea-ORM
- Prefer returning iterators over heap allocated values
- In `thiserror` enum variants, use `#[source]` instead of `#[from]`

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
- Run tests before marking any task done
- Run `cargo +nightly fmt` and `cargo clippy` after Rust changes; fix all warnings before finishing
- Run `dart format` after Dart changes
- After modifying the Flutter-Rust bridge API, regenerate bindings with the script above
- Mock-OIDC integration tests run serially (configured in `.config/nextest.toml`)
