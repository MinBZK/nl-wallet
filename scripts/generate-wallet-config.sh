#!/usr/bin/env bash

set -e # break on error
set -u # warn against undefined variables
set -o pipefail # break on error in pipeline

SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"
BASE_DIR="$(dirname "${SCRIPTS_DIR}")"

# Source: https://gist.github.com/stokito/f2d7ea0b300f14638a9063559384ec89
base64_padding()
{
  local len=$(( ${#1} % 4 ))
  local padded_b64=''
  if [ ${len} = 2 ]; then
    padded_b64="${1}=="
  elif [ ${len} = 3 ]; then
    padded_b64="${1}="
  else
    padded_b64="${1}"
  fi
  echo -n "$padded_b64"
}

base64_url_decode() { base64_padding "$1" | tr -- '-_' '+/' | openssl base64 -d -A; }

# Print usage instructions when there are not enough arguments.
if [ $# -lt 5 ]; then
    >&2 echo "Usage: $0 <output location> <wallet env> <env static hostname> <config public key> <config server TLS CA> [<config server TLS CA>..]"
    exit 1
fi

# Store arguments in variables, catching all remaining ones as CAs.
OUTPUT_LOCATION=$1
WALLET_ENV=$2
STATIC_HOSTNAME=$3
CONFIG_PUBLIC_KEY=$4
CONFIG_SERVER_CAS=("${@:5}")

# Create temp dir for intermediate files and clean on exit
trap clean EXIT
clean() {
    if [[ -n "${TEMP_OUTDIR:-}" ]]; then
        rm -rf "${TEMP_OUTDIR}"
    fi
}
TEMP_OUTDIR="$(mktemp -d)"

# Create a temporary PEM file with all of the CA certificates.
CA_FILE="${TEMP_OUTDIR}/ca.pem"

for CA in "${CONFIG_SERVER_CAS[@]}"; do
    {
        echo "-----BEGIN CERTIFICATE-----"
        echo "$CA"
        echo "-----END CERTIFICATE-----"
    } >> "$CA_FILE"
done

# Fetch the configuration and extract its payload.
CONFIG_JWT=$(curl -sf --cacert "$CA_FILE" "https://${STATIC_HOSTNAME}/config/v1/wallet-config")
JWT_PAYLOAD=$(echo "$CONFIG_JWT" | cut -d . -f 2)

# Generate the config_server_configuration JSON in the temp dir, so it can be used for verification below.
CONFIG_SERVER_CONFIG_FILE="${TEMP_OUTDIR}/config-server-config.json"
jq -n \
    '{
        "environment": $env,
        "http_config": {
            "base_url": $url,
            "trust_anchors": $ARGS.positional
        },
        "signing_public_key": $pubkey,
        "update_frequency_in_sec": 3600
    }' \
    --arg env "$WALLET_ENV" \
    --arg url "https://${STATIC_HOSTNAME}/config/v1/" \
    --arg pubkey "${CONFIG_PUBLIC_KEY}" \
    --args "${CONFIG_SERVER_CAS[@]}" \
    > "$CONFIG_SERVER_CONFIG_FILE"

# Verify the configuration, including its signature, using the same code and validation as the wallet.
CONFIG_JWT_FILE="${TEMP_OUTDIR}/wallet-config.jwt"
echo -n "$CONFIG_JWT" > "$CONFIG_JWT_FILE"
cargo run --manifest-path "${BASE_DIR}/wallet_core/Cargo.toml" --locked --package wallet_configuration \
    --features verify --bin verify_wallet_config -- "$CONFIG_SERVER_CONFIG_FILE" "$CONFIG_JWT_FILE"

# Output wallet_configuration JSON
base64_url_decode "$JWT_PAYLOAD" > "${OUTPUT_LOCATION}/wallet-config.json"

# Output config_server_configuration JSON
cp "$CONFIG_SERVER_CONFIG_FILE" "${OUTPUT_LOCATION}/config-server-config.json"
