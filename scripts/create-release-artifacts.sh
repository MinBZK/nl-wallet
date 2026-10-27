#!/usr/bin/env bash

set -euo pipefail

SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"
BASE_DIR="$(dirname "${SCRIPTS_DIR}")"

version="v$(yq -r .version < "${BASE_DIR}/wallet_app/pubspec.yaml")"

zip -j9 wallet-issuance-server_${version}_x86_64-linux-glibc.zip \
    "${BASE_DIR}/wallet_core/target/x86_64-unknown-linux-gnu/release/issuance_server" \
    "${BASE_DIR}/wallet_core/target/x86_64-unknown-linux-gnu/release/issuance_server_migrations"

zip -j9 wallet-issuance-server_${version}_x86_64-linux-musl.zip \
    "${BASE_DIR}/wallet_core/target/x86_64-unknown-linux-musl/release/issuance_server" \
    "${BASE_DIR}/wallet_core/target/x86_64-unknown-linux-musl/release/issuance_server_migrations"

zip -j9 wallet-verification-server_${version}_x86_64-linux-glibc.zip \
    "${BASE_DIR}/wallet_core/target/x86_64-unknown-linux-gnu/release/verification_server" \
    "${BASE_DIR}/wallet_core/target/x86_64-unknown-linux-gnu/release/verification_server_migrations"

zip -j9 wallet-verification-server_${version}_x86_64-linux-musl.zip \
    "${BASE_DIR}/wallet_core/target/x86_64-unknown-linux-musl/release/verification_server" \
    "${BASE_DIR}/wallet_core/target/x86_64-unknown-linux-musl/release/verification_server_migrations"

zip -j9 -MM wallet-sbom_${version}_generic.zip "${BASE_DIR}/bom.json" "${BASE_DIR}/bom.xml"
zip -j9 -MM wallet-sbom-prod_${version}_generic.zip "${BASE_DIR}/bom-prod.json" "${BASE_DIR}/bom-prod.xml"

zip -j9 wallet-web_${version}_generic.zip \
    "${BASE_DIR}/wallet_web/dist/nl-wallet-web.d.ts" \
    "${BASE_DIR}/wallet_web/dist/nl-wallet-web.iife.js" \
    "${BASE_DIR}/wallet_web/dist/nl-wallet-web.js" \
    "${BASE_DIR}/wallet_web/dist/nl-wallet-web.umd.cjs"

shasum -a 256 *.zip > SHA256SUMS
