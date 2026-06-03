#!/usr/bin/env bash

set -e # break on error
set -u # warn against undefined variables
set -o pipefail # break on error in pipeline

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

# Fetch the configuration and split the JWT into its constituent parts.
CONFIG_JWT=$(curl -sf --cacert "$CA_FILE" "https://${STATIC_HOSTNAME}/config/v1/wallet-config")
JWT_HEADER=$(echo "$CONFIG_JWT" | cut -d . -f 1)
JWT_PAYLOAD=$(echo "$CONFIG_JWT" | cut -d . -f 2)
JWT_SIGNATURE=$(echo "$CONFIG_JWT" | cut -d . -f 3)

# Store the provided public key in a temporary PEM file.
CONFIG_PUB_KEY_FILE="${TEMP_OUTDIR}/config.pub.pem"
{
    echo "-----BEGIN PUBLIC KEY-----"
    echo "$CONFIG_PUBLIC_KEY"
    echo "-----END PUBLIC KEY-----"
} >> "$CONFIG_PUB_KEY_FILE"

# Generate a temporary ASN1 configuration containing both the R and S values of the ECDSA signature, in hex.
CONFIG_SIGNATURE_ASN1_FILE="${TEMP_OUTDIR}/config_signature.asn1"

SIGNATURE_R=$(base64_url_decode "$JWT_SIGNATURE" | xxd -p -l 32 -c 32)
SIGNATURE_S=$(base64_url_decode "$JWT_SIGNATURE" | xxd -p -s 32 -c 32)
{
    echo "asn1=SEQUENCE:seq"
    echo "[seq]"
    echo "r=INTEGER:0x${SIGNATURE_R}"
    echo "s=INTEGER:0x${SIGNATURE_S}"
} > "$CONFIG_SIGNATURE_ASN1_FILE"

# Generate a temporary binary signature file in DER format.
CONFIG_SIGNATURE_FILE="${TEMP_OUTDIR}/config_signature.der"
openssl asn1parse -genconf "$CONFIG_SIGNATURE_ASN1_FILE" -out "$CONFIG_SIGNATURE_FILE" >/dev/null

# Actually verify the header and payload against the signature.
if ! echo -n "${JWT_HEADER}.${JWT_PAYLOAD}" | openssl dgst -sha256 -verify "$CONFIG_PUB_KEY_FILE" -signature "$CONFIG_SIGNATURE_FILE" >/dev/null; then
    >&2 echo "Configuration signature verification failed"
    exit 1
fi

# Output wallet_configuration JSON
base64_url_decode "$JWT_PAYLOAD" > "${OUTPUT_LOCATION}/wallet-config.json"

# Output config_server_configuratino JSON
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
    > "${OUTPUT_LOCATION}/config-server-config.json"
