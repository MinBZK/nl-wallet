#!/usr/bin/env bash

set -e # break on error
set -u # warn against undefined variables
set -o pipefail # break on error in pipeline

SCRIPTS_DIR="$(cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd)"
BASE_DIR=$(dirname "${SCRIPTS_DIR}")

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
    >&2 echo "Usage: $0 <wallet env> <env app hostname> <env static hostname> <config public key> <config server TLS CA> [<config server TLS CA>..]"
    exit 1
fi

# Store arguments in variables, catching all remaining ones as CAs.
WALLET_ENV=$1
APP_HOSTNAME=$2
STATIC_HOSTNAME=$3
CONFIG_PUBLIC_KEY=$4
CONFIG_SERVER_CAS=("${@:5}")

# Create a temporary PEM file with all of the CA certificates.
CA_FILE=$(mktemp --tmpdir "generate_wallet_env_file.config_server_ca.XXXXXXXXXX")
trap 'rm -f "$CA_FILE"' 0 2 3 15 # remove the file whenever the script exits.

for CA in "${CONFIG_SERVER_CAS[@]}"; do
    {
        echo "-----BEGIN CERTIFICATE-----"
        echo "$CA"
        echo "-----END CERTIFICATE-----"
    } >> "$CA_FILE"
done

# Fetch the configuration and split the JWT into its constituent parts.
CONFIG_JWT=$(curl -sf --cacert "$CA_FILE" "https://${STATIC_HOSTNAME}/config/v1/wallet-config")
JWT_HEADER=$(echo "$CONFIG_JWT" | cut -d . -f 1)
JWT_PAYLOAD=$(echo "$CONFIG_JWT" | cut -d . -f 2)
JWT_SIGNATURE=$(echo "$CONFIG_JWT" | cut -d . -f 3)

# Store the provided public key in a temporary PEM file.
CONFIG_PUB_KEY_FILE=$(mktemp --tmpdir "generate_wallet_env_file.config_public_key.XXXXXXXXXX")
trap 'rm -f "$CONFIG_PUB_KEY_FILE"' 0 2 3 15 # remove the file whenever the script exits.
{
    echo "-----BEGIN PUBLIC KEY-----"
    echo "$CONFIG_PUBLIC_KEY"
    echo "-----END PUBLIC KEY-----"
} >> "$CONFIG_PUB_KEY_FILE"

# Generate a temporary ASN1 configuration containing both the R and S values of the ECDSA signature, in hex.
CONFIG_SIGNATURE_ASN1_FILE=$(mktemp --tmpdir "generate_wallet_env_file.config_signature_asn1.XXXXXXXXXX")
trap 'rm -f "$CONFIG_SIGNATURE_ASN1_FILE"' 0 2 3 15 # remove the file whenever the script exits.

SIGNATURE_R=$(base64_url_decode "$JWT_SIGNATURE" | xxd -p -l 32 -c 32)
SIGNATURE_S=$(base64_url_decode "$JWT_SIGNATURE" | xxd -p -s 32 -c 32)
{
    echo "asn1=SEQUENCE:seq"
    echo "[seq]"
    echo "r=INTEGER:0x${SIGNATURE_R}"
    echo "s=INTEGER:0x${SIGNATURE_S}"
} > "$CONFIG_SIGNATURE_ASN1_FILE"

# Generate a temporary binary signature file in DER format.
CONFIG_SIGNATURE_FILE=$(mktemp --tmpdir "generate_wallet_env_file.config_signature.XXXXXXXXXX")
trap 'rm -f "$CONFIG_SIGNATURE_FILE"' 0 2 3 15 # remove the file whenever the script exits.

openssl asn1parse -genconf "$CONFIG_SIGNATURE_ASN1_FILE" -out "$CONFIG_SIGNATURE_FILE" >/dev/null

# Actually verify the header and payload against the signature.
if ! echo -n "${JWT_HEADER}.${JWT_PAYLOAD}" | openssl dgst -sha256 -verify "$CONFIG_PUB_KEY_FILE" -signature "$CONFIG_SIGNATURE_FILE" >/dev/null; then
    >&2 echo "Configuration signature verification failed"
    exit 1
fi

# Output wallet_configuration JSON
base64_url_decode "$JWT_PAYLOAD" > "${BASE_DIR}/../wallet_core/wallet/wallet-config.json"

# Output config_server_configuratino JSON
jq -n \
    --arg env "$WALLET_ENV" \
    --arg url "https://${STATIC_HOSTNAME}/config/v1/" \
    --arg ca "$(IFS="|" ; echo "${CONFIG_SERVER_CAS[*]}")" \
    --arg pubkey "${CONFIG_PUBLIC_KEY}" \
    --arg freq 3600 \
    '{"environment":$env,"http_config":{"base_url":$url,"trust_anchors":[$ca]},"signing_public_key":$pubkey,"update_frequency_in_sec":$freq}' \
    > "${BASE_DIR}/../wallet_core/wallet/config-server-config.json"

echo "UNIVERSAL_LINK_BASE=https://${APP_HOSTNAME}/deeplink/"
