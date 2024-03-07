#!/usr/bin/env bash

source "${SCRIPTS_DIR}/utils.sh"

WALLET_CORE_DIR="${BASE_DIR}/wallet_core"
WP_DIR="${WALLET_CORE_DIR}/wallet_provider"
MRP_WALLET_SERVER_DIR="${WALLET_CORE_DIR}/wallet_server"
MOCK_RELYING_PARTY_DIR="${WALLET_CORE_DIR}/mock_relying_party"
CS_DIR="${WALLET_CORE_DIR}/configuration_server"

DEVENV="${SCRIPTS_DIR}/devenv"
TARGET_DIR="${SCRIPTS_DIR}/devenv/target"

# source user variables
[ -f "${SCRIPTS_DIR}/.env" ] && . "${SCRIPTS_DIR}/.env"

# Path of the nl-rdo-max-private repository
export DIGID_CONNECTOR_PATH=${DIGID_CONNECTOR_PATH:-$(realpath "${BASE_DIR}"/../nl-rdo-max-private)}

# Set to `10.0.2.2` for android or to `localhost` for ios
# export SERVICES_HOST=10.0.2.2
export SERVICES_HOST=localhost

# Database properties for the wallet_provider, with defaults.
# The defaults will work when using the `wallet_core/wallet_provider/docker-compose.yml` file.
# Set these properties before executing this script
export DB_HOST="${DB_HOST:-localhost}"
export DB_USERNAME="${DB_USERNAME:-postgres}"
export DB_PASSWORD="${DB_PASSWORD:-postgres}"
export DB_NAME="${DB_NAME:-wallet_provider}"
export PGADMIN_DEFAULT_PASSWORD="${PGADMIN_DEFAULT_PASSWORD:-admin}}"

# HSM properties, with defaults
HSM_LIBRARY_PATH=$(detect_softhsm)
export HSM_LIBRARY_PATH
export HSM_SO_PIN=${HSM_SO_PIN:-12345678}
export HSM_USER_PIN=${HSM_USER_PIN:-12345678}
export DEFAULT_HSM_TOKEN_DIR="${HOME}/.softhsm2/tokens"
export HSM_TOKEN_DIR=${HSM_TOKEN_DIR:-$DEFAULT_HSM_TOKEN_DIR}

# export WALLET_CLIENT_ID=$(uuidgen)
export WALLET_CLIENT_ID=3e58016e-bc2e-40d5-b4b1-a3e25f6193b9

export RP_RETURN_URL="${RP_RETURN_URL:-http://${SERVICES_HOST}:3004/}" # default: http://${SERVICES_HOST}:3004/
