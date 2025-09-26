#!/usr/bin/env bash

set -euo pipefail

command="${1:-fresh}"

SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"

. $SCRIPTS_DIR/configuration.sh

DATABASE_URL="postgres://$DB_USERNAME:$DB_PASSWORD@$DB_HOST:$DB_PORT/issuance_server" \
    cargo run --manifest-path "$BASE_DIR/wallet_core/Cargo.toml" --bin issuance_server_migrations -- $command

DATABASE_URL="postgres://$DB_USERNAME:$DB_PASSWORD@$DB_HOST:$DB_PORT/pid_issuer" \
    cargo run --manifest-path "$BASE_DIR/wallet_core/Cargo.toml" --bin pid_issuer_migrations -- $command

DATABASE_URL="postgres://$DB_USERNAME:$DB_PASSWORD@$DB_HOST:$DB_PORT/verification_server" \
    cargo run --manifest-path "$BASE_DIR/wallet_core/Cargo.toml" --bin verification_server_migrations -- $command

cargo run --manifest-path "$BASE_DIR/wallet_core/Cargo.toml" --bin wallet_provider_migrations -- $command
