#!/usr/bin/env bash

set -euo pipefail

crate="${1:-}"

SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"

. $SCRIPTS_DIR/configuration.sh

if [[ -z $crate || $crate == 'server_utils' ]]; then
    rm -f "$BASE_DIR/wallet_core/wallet_server/server_utils/src/entity"/*
    sea-orm-cli generate entity \
        --database-url "postgres://$DB_USERNAME:$DB_PASSWORD@$DB_HOST:$DB_PORT/verification_server" \
        --output-dir "$BASE_DIR/wallet_core/wallet_server/server_utils/src/entity"
    cargo fmt --manifest-path "$BASE_DIR/wallet_core/wallet_server/server_utils/Cargo.toml"
fi

if [[ -z $crate || $crate == 'status_lists' ]]; then
    rm -f "$BASE_DIR/wallet_core/lib/status_lists/src/entity"/*
    sea-orm-cli generate entity \
        --database-url "postgres://$DB_USERNAME:$DB_PASSWORD@$DB_HOST:$DB_PORT/issuance_server" \
        --ignore-tables "seaql_migrations,session_state" \
        --output-dir "$BASE_DIR/wallet_core/lib/status_lists/src/entity"
    cargo fmt --manifest-path "$BASE_DIR/wallet_core/lib/status_lists/Cargo.toml"
fi

if [[ -z $crate || $crate == 'wallet_provider' ]]; then
    rm -f "$BASE_DIR/wallet_core/wallet_server/wallet_provider/src/entity"/*
    sea-orm-cli generate entity \
        --database-url "postgres://$DB_USERNAME:$DB_PASSWORD@$DB_HOST:$DB_PORT/wallet_provider" \
        --ignore-tables "seaql_migrations,attestation_batch,attestation_batch_list_indices,attestation_type,status_list,status_list_flag,status_list_item" \
        --output-dir "$BASE_DIR/wallet_core/wallet_provider/persistence/src/entity"
    cargo fmt --manifest-path "$BASE_DIR/wallet_core/wallet_provider/Cargo.toml"
fi
