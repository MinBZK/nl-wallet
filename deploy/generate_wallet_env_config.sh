#!/usr/bin/env bash

set -e # break on error
set -u # warn against undefined variables
set -o pipefail # break on error in pipeline

SCRIPTS_DIR=$(dirname "$(realpath "$(command -v "${BASH_SOURCE[0]}")")")

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

base64_url_decode() { base64_padding $1 | tr -- '-_' '+/' | openssl base64 -d -A; }

# Print usage instructions when there are not enough arguments.
if [ $# -lt 4 ]; then
    >&2 echo "Usage: $0 <env app hostname> <env static hostname> <config public key> <config server TLS CA> [<config server TLS CA>..]"
    exit 1
fi

# Store arguments in variables, catching all remaining ones as CAs.
APP_HOSTNAME=$1
STATIC_HOSTNAME=$2
CONFIG_PUBLIC_KEY=$3
CONFIG_SERVER_CAS=("${@:4}")

# Create a temporary PEM file with all of the CA certificates.
CA_FILE=$(mktemp --tmpdir "generate_wallet_env_file.config_server_ca.XXXXXXXXXX")
trap 'rm -f "$CA_FILE"' 0 2 3 15 # remove the file whenever the script exits.

for CA in "${CONFIG_SERVER_CAS[@]}"; do
    echo "-----BEGIN CERTIFICATE-----" >>"$CA_FILE"
    echo $CA >>"$CA_FILE"
    echo "-----END CERTIFICATE-----" >>"$CA_FILE"
done

# Fetch the configuration and split the JWT into its constituent parts.
CONFIG_JWT=$(curl -sf --cacert "$CA_FILE" "https://${STATIC_HOSTNAME}/config/v1/wallet-config")
JWT_HEADER=$(echo $CONFIG_JWT | cut -d . -f 1)
JWT_PAYLOAD=$(echo $CONFIG_JWT | cut -d . -f 2)
JWT_SIGNATURE=$(echo $CONFIG_JWT | cut -d . -f 3)

# Store the provided public key in a temporary PEM file.
CONFIG_PUB_KEY_FILE=$(mktemp --tmpdir "generate_wallet_env_file.config_public_key.XXXXXXXXXX")
trap 'rm -f "$CONFIG_PUB_KEY_FILE"' 0 2 3 15 # remove the file whenever the script exits.

echo "-----BEGIN PUBLIC KEY-----" >"$CONFIG_PUB_KEY_FILE"
echo $CONFIG_PUBLIC_KEY >>"$CONFIG_PUB_KEY_FILE"
echo "-----END PUBLIC KEY-----" >>"$CONFIG_PUB_KEY_FILE"

# Generate a temporary ASN1 configuration containing both the R and S values of the ECDSA signature, in hex.
CONFIG_SIGNATURE_ASN1_FILE=$(mktemp --tmpdir "generate_wallet_env_file.config_signature_asn1.XXXXXXXXXX")
trap 'rm -f "$CONFIG_SIGNATURE_ASN1_FILE"' 0 2 3 15 # remove the file whenever the script exits.

SIGNATURE_R=$(base64_url_decode $JWT_SIGNATURE | xxd -p -l 32 -c 32)
SIGNATURE_S=$(base64_url_decode $JWT_SIGNATURE | xxd -p -s 32 -c 32)

echo "asn1=SEQUENCE:seq" >"$CONFIG_SIGNATURE_ASN1_FILE"
echo "[seq]" >>"$CONFIG_SIGNATURE_ASN1_FILE"
echo "r=INTEGER:0x${SIGNATURE_R}" >>"$CONFIG_SIGNATURE_ASN1_FILE"
echo "s=INTEGER:0x${SIGNATURE_S}" >>"$CONFIG_SIGNATURE_ASN1_FILE"

# Generate a temporary binary signature file in DER format.
CONFIG_SIGNATURE_FILE=$(mktemp --tmpdir "generate_wallet_env_file.config_signature.XXXXXXXXXX")
trap 'rm -f "$CONFIG_SIGNATURE_FILE"' 0 2 3 15 # remove the file whenever the script exits.

openssl asn1parse -genconf "$CONFIG_SIGNATURE_ASN1_FILE" -out "$CONFIG_SIGNATURE_FILE" >/dev/null

# Actually verify the header and payload against the signature.
set +e
echo -n "${JWT_HEADER}.${JWT_PAYLOAD}" | openssl dgst -sha256 -verify "$CONFIG_PUB_KEY_FILE" -signature "$CONFIG_SIGNATURE_FILE" >/dev/null

if [ $? -ne 0 ]; then
    >&2 echo "Configuration signature verification failed"
    exit 1
fi
set -e

# Output the lines of the .env file based on the contents of the configuration JSON.
CONFIG_JSON=$(base64_url_decode $JWT_PAYLOAD)

echo "CONFIG_SERVER_BASE_URL=https://${STATIC_HOSTNAME}/config/v1/"
echo "CONFIG_SERVER_TRUST_ANCHORS=$(IFS="|" ; echo "${CONFIG_SERVER_CAS[*]}")"
echo "CONFIG_SERVER_SIGNING_PUBLIC_KEY=${CONFIG_PUBLIC_KEY}"
echo "UNIVERSAL_LINK_BASE=https://${APP_HOSTNAME}/deeplink/"

echo "WALLET_CONFIG_VERSION=$(echo $CONFIG_JSON | jq -r '.version' )"
echo "WALLET_PROVIDER_BASE_URL=$(echo $CONFIG_JSON | jq -r '.account_server.base_url' )"
echo "CERTIFICATE_PUBLIC_KEY=$(echo $CONFIG_JSON | jq -r '.account_server.certificate_public_key' )"
echo "INSTRUCTION_RESULT_PUBLIC_KEY=$(echo $CONFIG_JSON | jq -r '.account_server.instruction_result_public_key' )"
echo "WTE_PUBLIC_KEY=$(echo $CONFIG_JSON | jq -r '.account_server.wte_public_key' )"
echo "PID_ISSUER_URL=$(echo $CONFIG_JSON | jq -r '.pid_issuance.pid_issuer_url' )"
echo "DIGID_URL=$(echo $CONFIG_JSON | jq -r '.pid_issuance.digid_url' )"
echo "DIGID_CLIENT_ID=$(echo $CONFIG_JSON | jq -r '.pid_issuance.digid_client_id' )"
echo "DIGID_APP2APP_ENV=$(echo $CONFIG_JSON | jq -r '.pid_issuance.digid_app2app.env' )"
echo "DIGID_APP2APP_HOST=$(echo $CONFIG_JSON | jq -r '.pid_issuance.digid_app2app.host' )"
echo "DIGID_APP2APP_UNIVERSAL_LINK=$(echo $CONFIG_JSON | jq -r '.pid_issuance.digid_app2app.universal_link' )"

RP_TRUST_ANCHORS=($(echo $CONFIG_JSON | jq -r '.disclosure.rp_trust_anchors[]'))
echo "RP_TRUST_ANCHORS=$(IFS="|" ; echo "${RP_TRUST_ANCHORS[*]}")"

MDOC_TRUST_ANCHORS=($(echo $CONFIG_JSON | jq -r '.mdoc_trust_anchors[]'))
echo "MDOC_TRUST_ANCHORS=$(IFS="|" ; echo "${MDOC_TRUST_ANCHORS[*]}")"
