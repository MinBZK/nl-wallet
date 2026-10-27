# Generate full and production SBOMs

Run the standard generators used by the existing CI job:

```sh
bash scripts/sbom/generate.sh /tmp/nl-wallet-sbom
```

Use a new or empty output directory. The command produces `bom.json`, `bom.xml`,
`bom-prod.json` and `bom-prod.xml`, and validates all four with the CycloneDX CLI.
Both variants use CycloneDX 1.5.

The CycloneDX CLI merges the generator outputs hierarchically, preserving each
root's dependency graph and avoiding duplicate references from separate Cargo
SBOMs. Components are nested beneath their roots; the same package can occur
under several roots. Use package URLs when comparing unique dependencies.
Before merging, a small jq cleanup removes cdxgen's duplicate project entry from
`components` when it is already in `metadata.component`. This keeps every package
and dependency reference; it does not select production dependencies.

## Prerequisites

Use the CI quality image, or install the versions pinned in
`deploy/docker-images/ci/cyclonedx.sh`: cargo-cyclonedx 0.5.9, cdxgen 11.1.2,
cyclonedx-cocoapods 2.0.1 and cyclonedx-cli 0.30.0. The normal Cargo, pnpm,
Flutter, CocoaPods, Java and Android SDK setup is also required, along with jq.
Run `bundle install` for the repository's pinned CocoaPods version. The CocoaPods
generator must be available on PATH outside Bundler; it is installed in the CI
quality image. Configure `ANDROID_HOME` or `sdk.dir` as for an Android build.

Dependencies are resolved from the existing manifests/lockfiles. The script runs
`flutter pub get --enforce-lockfile` and `bundle exec pod install --deployment`, without
building or signing an app. Temporary Cargo SBOM files are removed on exit.

## Scope

| Ecosystem | Full SBOM | Production SBOM |
| --- | --- | --- |
| Rust | Existing cargo-cyclonedx behavior for the workspace and uniffi-bindgen. | `--no-build-deps`, merging outputs for the app, CI-built services and migrations, including demo services. |
| Web and admin portal | `pnpm sbom --lockfile-only`. | Add `--prod`. |
| Dart | cdxgen reads the workspace lockfile from the repository root. | **The same complete Dart SBOM, including development dependencies.** |
| Android | cdxgen's normal Gradle dependency collection. | Gradle `releaseRuntimeClasspath`; skip the standalone `:test-app` module. |
| iOS | cyclonedx-cocoapods. | Add `--exclude-test-targets`. |

The Dart exception is intentional and is recorded in the production SBOM's
metadata. The `-prod` file therefore does not mean that every development
dependency has been eliminated.

Production selection follows each generator's native semantics. In particular,
cargo-cyclonedx already omits Cargo dev-dependencies from both variants;
`--no-build-deps` does not remove procedural macros declared as normal
dependencies. Cargo uses the host target and default features, as in the existing
CI job. pnpm retains production packages' optional and peer dependencies.
CocoaPods test exclusion is based on target names, not binary inspection.
cdxgen retains a `:test-app` module descriptor even when its dependencies are
skipped; that descriptor is not a collected test dependency tree.

As in the existing CI job, CocoaPods collection enables Flutter's SwiftPM flag
so SwiftPM-compatible plugins are not also included as pods. The original
generated setting is restored on exit. Swift Package Manager dependencies
remain outside this collector's coverage. The existing removal of the placeholder
Flutter pod version `1.0.0` is retained.

## CI integration

`build-sbom` in `deploy/gitlab/sbom.yml` runs this generator automatically on the
default branch and is available as a manual job in merge-request pipelines.
Both JSON files are CycloneDX reports, and all four files are downloadable job
artifacts. On the default branch, the full files are stored at
`qt/quality-time/sbom/` and the production files at `qt/quality-time/sbom-prod/`.

Two default-branch jobs upload the JSON files to Dependency-Track:

| Job | File | Project |
| --- | --- | --- |
| `upload-sbom-to-dependency-track` | `bom.json` | Existing `project_id` from `nl-wallet-dependency-track-secret`. |
| `upload-prod-sbom-to-dependency-track` | `bom-prod.json` | `bf56a989-fdcc-4316-a512-533fee3ad8c0`, configured as `DEPENDENCY_TRACK_PROJECT_ID`. |

Both jobs reuse the existing secret's API key and upload URL. The API key must
have upload access to both projects. No project is created by the pipeline.
The build and upload jobs retain the existing `allow_failure: true` behavior.

The release job downloads both variants and creates `wallet-sbom_<version>_generic.zip`
and `wallet-sbom-prod_<version>_generic.zip`, each containing its JSON and XML files.
Both archives are included in `SHA256SUMS`.

The generator itself only creates local files. The earlier `sbom-comparison/`
prototype files are not inputs to this script or the CI job.
