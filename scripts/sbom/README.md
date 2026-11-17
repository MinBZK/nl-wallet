# SBOM generation

`generate.sh` collects dependencies from Rust, web, Dart, Android and iOS
and produces two CycloneDX 1.5 inventories:

- `bom.json` and `bom.xml`: the generators' default dependency sets.
- `bom-prod.json` and `bom-prod.xml`: production dependencies, using each
  generator's filters and selected Rust application/service roots.

The production Rust roots are `flutter_api`, `wallet_provider`,
`wallet_provider_migrations`, `audit_log_migrations`, `update_policy_server`,
and `revocation_portal`. Demo applications, issuer/verifier services and their
migration binaries, and `gba_hc_converter` opt out via
`package.metadata.sbom.production-root = false`. These packages remain in the
full inventory; dependencies also required by a production root remain in the
production inventory.

For Flutter, `dart_production.py` follows `directDependencies` from the `wallet`
package in `flutter pub deps --json`. It retains runtime dependencies transitively,
including local workspace packages and packages also used by development tools.
Development-only packages and unrelated workspace packages are excluded from the
production inventory. All Dart dependencies remain in the full inventory. A
missing runtime package or version mismatch fails generation instead of silently
omitting a dependency.

Swift Package Manager dependencies are not collected.

Run from the repository root with a new or empty output directory:

```sh
bash scripts/sbom/generate.sh /tmp/nl-wallet-sbom
```

Use the CI quality image, or your normal development setup plus the
[SBOM tools](../../deploy/docker-images/ci/cyclonedx.sh), jq and Python 3.9+.

`normalize.py` makes package references consistent across inputs and combines
duplicate components. It uses only the Python standard library. All four output
files are validated before being copied to the output directory.

[CI](../../deploy/gitlab/sbom.yml) uploads the two inventories to separate
Dependency-Track projects. Both are also included in release artifacts.
