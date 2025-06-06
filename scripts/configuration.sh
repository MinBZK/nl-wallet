#!/usr/bin/env bash

source "${SCRIPTS_DIR}/utils.sh"

WALLET_CORE_DIR="${BASE_DIR}/wallet_core"
WP_DIR="${WALLET_CORE_DIR}/wallet_provider"
PID_ISSUER_DIR="${WALLET_CORE_DIR}/wallet_server/pid_issuer"
ISSUANCE_SERVER_DIR="${WALLET_CORE_DIR}/wallet_server/issuance_server"
VERIFICATION_SERVER_DIR="${WALLET_CORE_DIR}/wallet_server/verification_server"
WALLET_WEB_DIR="${BASE_DIR}/wallet_web"
DEMO_RELYING_PARTY_DIR="${WALLET_CORE_DIR}/demo/demo_relying_party"
DEMO_ISSUER_DIR="${WALLET_CORE_DIR}/demo/demo_issuer"
DEMO_INDEX_DIR="${WALLET_CORE_DIR}/demo/demo_index"
CS_DIR="${WALLET_CORE_DIR}/configuration_server"
UPS_DIR="${WALLET_CORE_DIR}/update_policy/server"
GBA_HC_CONVERTER_DIR="${WALLET_CORE_DIR}/gba_hc_converter"

DEVENV="${SCRIPTS_DIR}/devenv"
TARGET_DIR="${SCRIPTS_DIR}/devenv/target"

# source user variables
[ -f "${SCRIPTS_DIR}/.env" ] && . "${SCRIPTS_DIR}/.env"

# Path and repository of the nl-rdo-max repository
export DIGID_CONNECTOR_PATH=${DIGID_CONNECTOR_PATH:-"${BASE_DIR}"/nl-rdo-max}
DIGID_CONNECTOR_REPOSITORY="https://github.com/minvws/nl-rdo-max.git"
DIGID_CONNECTOR_BASE_TAG="v2.11.0"
DIGID_CONNECTOR_BASE_COMMIT="e6daa09f94efe434c62e9617bd768fd909174a41"

# Set to `10.0.2.2` for android or to `localhost` for ios
# export SERVICES_HOST=10.0.2.2
export SERVICES_HOST=localhost

export WALLET_PROVIDER_PORT=3000
export CONFIG_SERVER_PORT=3001
export UPDATE_POLICY_SERVER_PORT=3002

export PID_ISSUER_WS_PORT=3003

export DEMO_INDEX_PORT=3004

export DEMO_ISSUER_PORT=3005
export DEMO_ISSUER_IS_PORT=3006
export ISSUANCE_SERVER_WS_PORT=3007

export DEMO_RP_PORT=3008
export VERIFICATION_SERVER_WS_PORT=3009
export VERIFICATION_SERVER_RS_PORT=3010

export BRP_SERVER_PORT=3011
export GBA_HC_CONV_PORT=3012

export RDO_MAX_PORT=8006

PID_ISSUER_API_KEY=$(echo $RANDOM | shasum -a1 | head -c 40)
export PID_ISSUER_API_KEY
GBA_HC_CONV_ENCRYPTION_KEY=$(openssl rand -hex 32)
export GBA_HC_CONV_ENCRYPTION_KEY
GBA_HC_CONV_HMAC_KEY=$(openssl rand -hex 64)
export GBA_HC_CONV_HMAC_KEY

# Google Cloud project number for Play Integrity. This number is not secret. In
# our case, this is currently a test throw-away project on one of our Google
# Cloud accounts. Note that this project is for testing purposes only, i.e., it
# is not intended for production use.
export GOOGLE_CLOUD_PROJECT_NUMBER=12143997365

# Database properties for the wallet_provider, with defaults.
# The defaults will work when using the `wallet_core/wallet/wallet_provider/docker-compose.yml` file.
# Set these properties before executing this script
export DB_HOST="${DB_HOST:-localhost}"
export DB_USERNAME="${DB_USERNAME:-postgres}"
export DB_PASSWORD="${DB_PASSWORD:-postgres}"
export DB_NAME="${DB_NAME:-wallet_provider}"
export PGADMIN_DEFAULT_PASSWORD="${PGADMIN_DEFAULT_PASSWORD:-admin}"

# HSM properties, with defaults
export HSM_LIBRARY_PATH="${HSM_LIBRARY_PATH:-$(detect_softhsm)}"
export HSM_SO_PIN=${HSM_SO_PIN:-12345678}
export HSM_USER_PIN=${HSM_USER_PIN:-12345678}
export DEFAULT_HSM_TOKEN_DIR="${HOME}/.softhsm2/tokens"
export HSM_TOKEN_DIR=${HSM_TOKEN_DIR:-$DEFAULT_HSM_TOKEN_DIR}

# export WALLET_CLIENT_ID=$(uuidgen)
export WALLET_CLIENT_ID=3e58016e-bc2e-40d5-b4b1-a3e25f6193b9

export SENTRY_ENVIRONMENT=${SENTRY_ENVIRONMENT:-local}
