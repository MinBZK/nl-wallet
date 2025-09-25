#!/usr/bin/env bash

set -euo pipefail

crate="${1:-}"

SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"

. $SCRIPTS_DIR/configuration.sh


if [[ -z $crate || $crate == 'server_utils' ]]; then
    sea-orm-cli generate entity \
        --database-url "postgres://$DB_USERNAME:$DB_PASSWORD@$DB_HOST:$DB_PORT/verification_server" \
        --output-dir "$BASE_DIR/wallet_core/wallet_server/server_utils/src/entity"
fi

if [[ -z $crate || $crate == 'wallet_provider' ]]; then
    sea-orm-cli generate entity \
        --database-url "postgres://$DB_USERNAME:$DB_PASSWORD@$DB_HOST:$DB_PORT/wallet_provider" \
        --output-dir "$BASE_DIR/wallet_core/wallet_provider/persistence/src/entity"
fi
