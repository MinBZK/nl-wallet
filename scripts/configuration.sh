#!/usr/bin/env bash

WP_DIR="${BASE_DIR}/wallet_core/wallet_provider"
PID_ISSUER_DIR="${BASE_DIR}/wallet_core/pid_issuer"

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
export DB_HOST="${DB_HOST:-localhost}" # default: localhost
export DB_USERNAME="${DB_USERNAME:-postgres}" # default: postgres
export DB_PASSWORD="${DB_PASSWORD:-postgres}" # default: postgres
export DB_NAME="${DB_NAME:-wallet_provider}" # default: wallet_provider

# export WALLET_CLIENT_ID=$(uuidgen)
export WALLET_CLIENT_ID=3e58016e-bc2e-40d5-b4b1-a3e25f6193b9

source "${SCRIPTS_DIR}/colors.sh"

SECTION=${LIGHT_BLUE}
SUCCESS=${LIGHT_GREEN}
INFO=${PURPLE}
