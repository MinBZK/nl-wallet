#!/usr/bin/env bash
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
output="${1:?Usage: bash scripts/sbom/generate.sh OUTPUT_DIRECTORY}"
mkdir -p "$output"
output="$(cd "$output" && pwd)"
if [[ -n "$(find "$output" -mindepth 1 -maxdepth 1 -print -quit)" ]]; then
    echo "Output directory must be empty: $output" >&2
    exit 1
fi
work=""
rust_files=()
cleanup() {
    if [[ -z "$work" ]]; then
        return
    fi
    if [[ -f "$work/flutter-plugins-dependencies" ]]; then
        cp "$work/flutter-plugins-dependencies" "$repo/wallet_app/.flutter-plugins-dependencies"
    fi
    if [[ ${#rust_files[@]} -gt 0 ]]; then
        rm -f "${rust_files[@]}"
    fi
    rm -rf "$work"
}
trap cleanup EXIT

work="$(mktemp -d)"
# cargo-cyclonedx writes one file next to each workspace member's manifest.
# Unique filenames keep previous SBOMs out of both merges and out of cleanup.
rust_name="sbom-$(basename "$work")"
full=()
prod=()
cd "$repo"

normalize_bom() {
    python3 "$repo/scripts/sbom/normalize.py" "$1" > "$work/normalized.json"
    mv "$work/normalized.json" "$1"
}

cargo metadata --locked --no-deps --format-version 1 --manifest-path wallet_core/Cargo.toml > "$work/rust.json"
cargo metadata --locked --no-deps --format-version 1 --manifest-path wallet_core/uniffi-bindgen/Cargo.toml > "$work/uniffi.json"
version=$(jq -er '.packages[] | select(.name == "flutter_api") | .version' "$work/rust.json")
jq -r '.packages[].manifest_path' "$work/rust.json" "$work/uniffi.json" > "$work/manifests"
while IFS= read -r manifest; do
    rust_files+=("$(dirname "$manifest")/$rust_name-full.json" "$(dirname "$manifest")/$rust_name-prod.json")
    full+=("$(dirname "$manifest")/$rust_name-full.json")
done < "$work/manifests"
for manifest in wallet_core/Cargo.toml wallet_core/uniffi-bindgen/Cargo.toml; do
    cargo cyclonedx --spec-version 1.5 --manifest-path "$manifest" --format json --override-filename "$rust_name-full"
done
cargo cyclonedx --spec-version 1.5 --manifest-path wallet_core/Cargo.toml --format json \
    --no-build-deps --override-filename "$rust_name-prod"

# Product roots, including the demo services already covered by the existing job.
# Standalone tests, developer tools and code generators are not production roots.
for package in flutter_api wallet_provider wallet_provider_migrations audit_log_migrations \
    update_policy_server pid_issuer pid_issuer_migrations issuance_server issuance_server_migrations \
    verification_server verification_server_migrations pacf_issuance_server pacf_issuance_server_migrations \
    acf_demo_issuer acf_demo_issuer_migrations revocation_portal gba_hc_converter \
    demo_issuer demo_relying_party demo_index; do
    manifest=$(jq -er --arg name "$package" '.packages[] | select(.name == $name) | .manifest_path' "$work/rust.json")
    prod+=("$(dirname "$manifest")/$rust_name-prod.json")
done

for project in wallet_web wallet_admin_portal; do
    (cd "$project" && pnpm sbom --sbom-format cyclonedx --sbom-spec-version 1.5 --lockfile-only) > "$work/$project.json"
    (cd "$project" && pnpm sbom --sbom-format cyclonedx --sbom-spec-version 1.5 --lockfile-only --prod) > "$work/$project-prod.json"
    full+=("$work/$project.json")
    prod+=("$work/$project-prod.json")
done

(cd wallet_app && flutter pub get --enforce-lockfile)
# The Dart workspace lockfile lives at the repository root. Include all Dart deps in BOTH variants.
cdxgen --fail-on-error --spec-version 1.5 --type dart --output "$work/dart.json"
full+=("$work/dart.json")
prod+=("$work/dart.json")

# Match the existing CI setup for CocoaPods, restoring Flutter's generated setting on exit.
cp wallet_app/.flutter-plugins-dependencies "$work/flutter-plugins-dependencies"
jq '.swift_package_manager_enabled.ios = true' "$work/flutter-plugins-dependencies" > wallet_app/.flutter-plugins-dependencies
(cd wallet_app/ios && bundle exec pod install --allow-root --deployment)
for variant in full prod; do
    flags=()
    if [[ "$variant" == prod ]]; then flags+=(--exclude-test-targets); fi
    # Bash 3.2 (macOS) treats empty arrays as unset under set -u.
    # The guarded expansion passes no arguments when flags is empty.
    (cd wallet_app/ios && cyclonedx-cocoapods ${flags[@]+"${flags[@]}"} --name nl-wallet-ios \
        --version "$version" --type application --output "$work/ios-$variant.xml")
    # Strip Flutter 1.0.0: this placeholder pod version does not identify a Flutter release.
    # https://github.com/flutter/flutter/issues/154023#issuecomment-2310710611
    # https://github.com/flutter/flutter/issues/85820#issuecomment-874103549
    cyclonedx convert --output-version v1_5 --input-file "$work/ios-$variant.xml" --output-format json \
        | jq --arg flutter 'pkg:cocoapods/Flutter@1.0.0?file_name=Flutter' \
            'del(.components[] | select(."bom-ref" == $flutter)) |
             del(.dependencies[]? | select(.ref == $flutter)) |
             del(.dependencies[].dependsOn[]? | select(. == $flutter))' > "$work/ios-$variant.json"
done
full+=("$work/ios-full.json")
prod+=("$work/ios-prod.json")

for project in wallet_app/android/app wallet_core/wallet/platform_support/android; do
    name=$(echo "$project" | tr / -)
    (cd "$project" && GRADLE_CMD=../gradlew cdxgen --fail-on-error --spec-version 1.5 --type gradle --output "$work/$name.json")
    (cd "$project" && GRADLE_CMD=../gradlew GRADLE_ARGS_DEPENDENCIES='--configuration releaseRuntimeClasspath' \
        GRADLE_SKIP_MODULES=:test-app cdxgen --fail-on-error --spec-version 1.5 --type gradle --output "$work/$name-prod.json")
    full+=("$work/$name.json")
    prod+=("$work/$name-prod.json")
done

# Normalize each input's local references before combining dependency graphs.
while IFS= read -r bom; do
    normalize_bom "$bom"
done < <(printf '%s\n' "${full[@]}" "${prod[@]}" | sort -u)

# Flat merging keeps shared dependencies once instead of repeating each crate's tree.
cyclonedx merge --name nl-wallet --version "$version" --output-version v1_5 \
    --input-files "${full[@]}" --output-file "$work/bom.json"
normalize_bom "$work/bom.json"
cyclonedx merge --name nl-wallet --version "$version" --output-version v1_5 \
    --input-files "${prod[@]}" --output-file "$work/merged-prod.json"
normalize_bom "$work/merged-prod.json"
jq '.metadata.properties += [{"name":"nl-wallet:sbom:dart-dependencies",
    "value":"All Dart dependencies are included, including development dependencies."}]' \
    "$work/merged-prod.json" > "$work/bom-prod.json"
for name in bom bom-prod; do
    cyclonedx convert --output-version v1_5 --input-file "$work/$name.json" --output-file "$work/$name.xml"
    echo "Validating $name.xml"
    cyclonedx validate --input-file "$work/$name.xml" --input-version v1_5 --fail-on-errors
    echo "Validating $name.json"
    cyclonedx validate --input-file "$work/$name.json" --input-version v1_5 --fail-on-errors
done
cp "$work/bom.json" "$work/bom.xml" "$work/bom-prod.json" "$work/bom-prod.xml" "$output/"
echo "SBOMs written to $output (production SBOM includes all Dart dependencies)"
