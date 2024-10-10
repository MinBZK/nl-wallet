#!/usr/bin/env bash
# This script generates configuration files, keys and certificates for a local development environment for the Wallet.
# It will configure the following applications:
#
# - nl-rdo-max (digid-connector)
# - wallet_provider
# - wallet
# - wallet_server
# - mock_relying_party
# - softhsm2
#
# User specific variables can be supplied in the `.env` files.
#
# Prerequisites:
#
# - cargo: needed to build/run wallet_provider_configuration
# - openssl: needed to generate keys and certificates
# - jq: needed to parse JSON
# - standard unix tools like: grep, sed, tr, ...
# - docker: with compose v2 extension, to run the digid-connector
# - softhsm2-util: software implementation of a hardware security module (HSM). See https://github.com/opendnssec/SoftHSMv2.
# - p11tool: utility that is part of the gnutls package. The Homebrew package is 'gnutls'. On Debian/Ubuntu, it is 'gnutls-bin'.
# - nodejs: JavaScript runtime environment for building frontend library
#
# MacOS specific instructions
# This script needs GNU sed. this can be installed by
#
#     brew install gnu-sed
#
# Before running this script
# - Start postgresql
#
#   This can be either an own managed instance, or one can use our docker compose script:
#
#     docker compose -f wallet_core/wallet_provider/docker-compose.yml up
#
# - Android Emulator configuration
#
#     ./scripts/map_android_ports.sh
#

set -e # break on error
set -u # warn against undefined variables
set -o pipefail
# set -x # echo statements before executing, useful while debugging

SCRIPTS_DIR=$(dirname "$(realpath "$(command -v "${BASH_SOURCE[0]}")")")
export SCRIPTS_DIR
BASE_DIR=$(dirname "${SCRIPTS_DIR}")
export BASE_DIR

source "${SCRIPTS_DIR}/utils.sh"

########################################################################
# Check prerequisites

expect_command cargo "Missing binary 'cargo', please install the Rust toolchain"
expect_command openssl "Missing binary 'openssl', please install OpenSSL"
expect_command jq "Missing binary 'jq', please install"
expect_command tr "Missing binary 'tr', please install"
expect_command xxd "Missing binary 'xxd', please install"
if [[ -z "${SKIP_DIGID_CONNECTOR:-}" ]]; then
    expect_command docker "Missing binary 'docker', please install Docker (Desktop)"
fi
expect_command softhsm2-util "Missing binary 'softhsm2-util', please install softhsm2"
expect_command p11tool "Missing binary 'p11tool', please install 'gnutls' using Homebrew on macOS or 'gnutls-bin' on Debian/Ubuntu."
if [[ -z "${SKIP_WALLET_WEB:-}" ]]; then
    expect_command node "Missing binary 'node', please install (e.g. using node version manager: nvm)"
fi
check_openssl

if is_macos
then
    expect_command gsed "Missing binary 'gsed', please install gnu-sed"
    GNUSED="gsed"
else
    GNUSED="sed"
fi

BASE64="openssl base64 -e -A"
base64_url_encode() { ${BASE64} | tr '/+' '_-' | tr -d '=\n'; }

########################################################################
# Configuration
########################################################################

source "${SCRIPTS_DIR}"/configuration.sh

if [ ! -f "${SCRIPTS_DIR}/.env" ]
then
    echo -e "${INFO}Saving initial environment variables${NC}"
    echo -e \
"#!/usr/bin/env bash
# export DIGID_CONNECTOR_PATH=${DIGID_CONNECTOR_PATH}
# export DB_HOST=${DB_HOST}
# export DB_USERNAME=${DB_USERNAME}
# export DB_PASSWORD=${DB_PASSWORD}
# export DB_NAME=${DB_NAME}
# export HSM_LIBRARY_PATH=${HSM_LIBRARY_PATH}
# export HSM_SO_PIN=${HSM_SO_PIN}
# export HSM_USER_PIN=${HSM_USER_PIN}
# export HSM_TOKEN_DIR=${HSM_TOKEN_DIR}" \
    > "${SCRIPTS_DIR}/.env"
    echo -e "${INFO}Your customizations are saved in ${CYAN}${SCRIPTS_DIR}/.env${NC}."
    echo -e "${INFO}Edit this file to reflect your environment, and un-comment the relevant variables.${NC}"
    echo -e "${INFO}Note that variables in this file can no longer be overridden by the command line.${NC}"
fi

########################################################################
# Main
########################################################################

mkdir -p "${TARGET_DIR}"
mkdir -p "${TARGET_DIR}/configuration_server"
mkdir -p "${TARGET_DIR}/pid_issuer"
mkdir -p "${TARGET_DIR}/mock_relying_party"
mkdir -p "${TARGET_DIR}/wallet_provider"

########################################################################
# Configure digid-connector

if [[ -z "${SKIP_DIGID_CONNECTOR:-}" ]]; then
  echo -e "${SECTION}Configure and start digid-connector${NC}"

  # Check for existing nl-rdo-max, re-use if existing, clone if not
  if [ -d "${DIGID_CONNECTOR_PATH}" ]; then
    echo -e "${INFO}Using existing nl-rdo-max repository (not cloning)${NC}"
  else
    echo -e "${INFO}Cloning nl-rdo-max repository: ${DIGID_CONNECTOR_PATH}${NC}"
    # Unfortunately we can't directly clone a commit hash, so clone the tag and reset to the commit
    git clone --depth 1 -b "${DIGID_CONNECTOR_BASE_TAG}" "${DIGID_CONNECTOR_REPOSITORY}" "${DIGID_CONNECTOR_PATH}"
  fi

  # Enter nl-rdo-max git repository
  cd "${DIGID_CONNECTOR_PATH}"

  # Checkout validated-working commit
  echo -e "${INFO}Switching to commit: ${DIGID_CONNECTOR_BASE_COMMIT}${NC}"
  git checkout -q "${DIGID_CONNECTOR_BASE_COMMIT}"

  # Apply the patches, if not applied before
  for p in "${BASE_DIR}/scripts/devenv/digid-connector/patches"/*; do
    if git apply --check "$p" 2> /dev/null; then
      echo -e "${INFO}Applying patch: $p${NC}"
      git apply "$p"
    else
      echo -e "${INFO}Skipping previously applied patch: $p${NC}"
    fi
  done

  make setup-secrets setup-saml setup-config

  render_template "${DEVENV}/digid-connector/max.conf" "${DIGID_CONNECTOR_PATH}/max.conf"
  render_template "${DEVENV}/digid-connector/clients.json" "${DIGID_CONNECTOR_PATH}/clients.json"
  render_template "${DEVENV}/digid-connector/login_methods.json" "${DIGID_CONNECTOR_PATH}/login_methods.json"

  generate_ssl_key_pair_with_san "${DIGID_CONNECTOR_PATH}/secrets/ssl" server "${DIGID_CONNECTOR_PATH}/secrets/cacert.crt" "${DIGID_CONNECTOR_PATH}/secrets/cacert.key"
  openssl x509 -in "${DIGID_CONNECTOR_PATH}/secrets/cacert.crt" \
          -outform der -out "${DIGID_CONNECTOR_PATH}/secrets/cacert.der"
  # shellcheck disable=SC2089
  DIGID_CA_CRT="\"$(< "${DIGID_CONNECTOR_PATH}/secrets/cacert.der" ${BASE64})\""
  # shellcheck disable=SC2090
  export DIGID_CA_CRT

  # Build max docker container
  docker compose build max
  # Generate JWK from private RSA key of test_client.
  CLIENT_PRIVKEY_JWK=$(docker compose run --rm max make --silent create-jwk)
  # Remove the 'kid' json field, because the digid-connector does not send a JWE 'kid' header claim, which is required
  # if `kid` field is specified.
  BSN_PRIVKEY=$(echo "${CLIENT_PRIVKEY_JWK}" | jq -c 'del(.kid)')
  export BSN_PRIVKEY
fi

########################################################################
# Build wallet_web frontend library

if [[ -z "${SKIP_WALLET_WEB:-}" ]]; then
    echo
    echo -e "${SECTION}Build wallet_web frontend${NC}"

    cd "${WALLET_WEB_DIR}"

    if [[ -n "${RM_OLD_WALLET_WEB+x}" ]]; then
        rm ../wallet_core/mock_relying_party/assets/*.iife.js || true
    fi

    VITE_HELP_BASE_URL=${VITE_HELP_BASE_URL:-http://$SERVICES_HOST}
    export VITE_HELP_BASE_URL
    npm ci && npm run build
    WALLET_WEB_SHA256=$(cat dist/nl-wallet-web.iife.js | openssl sha256 -binary | ${BASE64})
    export WALLET_WEB_SHA256
    WALLET_WEB_SHA256_FILENAME=$(cat dist/nl-wallet-web.iife.js | openssl sha256 -binary | base64_url_encode) # url safe to prevent '/' to appear in filename
    export WALLET_WEB_SHA256_FILENAME
    WALLET_WEB_FILENAME="nl-wallet-web.${WALLET_WEB_SHA256_FILENAME}.iife.js"
    export WALLET_WEB_FILENAME
    cp dist/nl-wallet-web.iife.js ../wallet_core/mock_relying_party/assets/${WALLET_WEB_FILENAME}
fi


########################################################################
# Configure wallet_server and mock_relying_party

echo
echo -e "${SECTION}Configure wallet_server and mock_relying_party${NC}"

cd "${BASE_DIR}"

# Generate root CA for cose signing
if [ ! -f "${TARGET_DIR}/pid_issuer/ca.key.pem" ]; then
    generate_pid_issuer_root_ca
else
    echo -e "${INFO}Target file '${TARGET_DIR}/pid_issuer/ca.key.pem' already exists, not (re-)generating PID root CA"
fi
openssl x509 -in "${TARGET_DIR}/pid_issuer/ca.crt.pem" \
        -outform der -out "${TARGET_DIR}/pid_issuer/ca_cert.der"

# Generate pid issuer key and cert
generate_pid_issuer_key_pair

PID_CA_CRT=$(< "${TARGET_DIR}/pid_issuer/ca_cert.der" ${BASE64})
export PID_CA_CRT
PID_ISSUER_KEY=$(< "${TARGET_DIR}/pid_issuer/issuer_key.der" ${BASE64})
export PID_ISSUER_KEY
PID_ISSUER_CRT=$(< "${TARGET_DIR}/pid_issuer/issuer_crt.der" ${BASE64})
export PID_ISSUER_CRT

# Generate MRP root CA
if [ ! -f "${TARGET_DIR}/mock_relying_party/ca.key.pem" ]; then
    generate_mock_relying_party_root_ca
else
    echo -e "${INFO}Target file '${TARGET_DIR}/mock_relying_party/ca.key.pem' already exists, not (re-)generating root CA"
fi
openssl x509 -in "${TARGET_DIR}/mock_relying_party/ca.crt.pem" \
        -outform der -out "${TARGET_DIR}/mock_relying_party/ca.crt.der"

# Generate CA for RPs
RP_CA_CRT=$(< "${TARGET_DIR}/mock_relying_party/ca.crt.der" ${BASE64})
export RP_CA_CRT

# Generate relying party key and cert
generate_mock_relying_party_key_pair mijn_amsterdam
MOCK_RELYING_PARTY_KEY_MIJN_AMSTERDAM=$(< "${TARGET_DIR}/mock_relying_party/mijn_amsterdam.key.der" ${BASE64})
export MOCK_RELYING_PARTY_KEY_MIJN_AMSTERDAM
MOCK_RELYING_PARTY_CRT_MIJN_AMSTERDAM=$(< "${TARGET_DIR}/mock_relying_party/mijn_amsterdam.crt.der" ${BASE64})
export MOCK_RELYING_PARTY_CRT_MIJN_AMSTERDAM

# Generate relying party key and cert
generate_mock_relying_party_key_pair online_marketplace
MOCK_RELYING_PARTY_KEY_ONLINE_MARKETPLACE=$(< "${TARGET_DIR}/mock_relying_party/online_marketplace.key.der" ${BASE64})
export MOCK_RELYING_PARTY_KEY_ONLINE_MARKETPLACE
MOCK_RELYING_PARTY_CRT_ONLINE_MARKETPLACE=$(< "${TARGET_DIR}/mock_relying_party/online_marketplace.crt.der" ${BASE64})
export MOCK_RELYING_PARTY_CRT_ONLINE_MARKETPLACE

# Generate relying party key and cert
generate_mock_relying_party_key_pair xyz_bank
MOCK_RELYING_PARTY_KEY_XYZ_BANK=$(< "${TARGET_DIR}/mock_relying_party/xyz_bank.key.der" ${BASE64})
export MOCK_RELYING_PARTY_KEY_XYZ_BANK
MOCK_RELYING_PARTY_CRT_XYZ_BANK=$(< "${TARGET_DIR}/mock_relying_party/xyz_bank.crt.der" ${BASE64})
export MOCK_RELYING_PARTY_CRT_XYZ_BANK

# Generate relying party key and cert
generate_mock_relying_party_key_pair monkey_bike
MOCK_RELYING_PARTY_KEY_MONKEY_BIKE=$(< "${TARGET_DIR}/mock_relying_party/monkey_bike.key.der" ${BASE64})
export MOCK_RELYING_PARTY_KEY_MONKEY_BIKE
MOCK_RELYING_PARTY_CRT_MONKEY_BIKE=$(< "${TARGET_DIR}/mock_relying_party/monkey_bike.crt.der" ${BASE64})
export MOCK_RELYING_PARTY_CRT_MONKEY_BIKE

if [[ -z "${SKIP_MOCK_RELYING_PARTY:-}" ]]; then
    WALLET_WEB_FILENAME="${WALLET_WEB_FILENAME:-nl-wallet-web.iife.js}"
    export WALLET_WEB_FILENAME
    WALLET_WEB_SHA256="${WALLET_WEB_SHA256:-$(cat ../wallet_core/mock_relying_party/assets/${WALLET_WEB_FILENAME} | openssl sha256 -binary | ${BASE64})}"
    export WALLET_WEB_SHA256
    render_template "${DEVENV}/mock_relying_party.toml.template" "${MOCK_RELYING_PARTY_DIR}/mock_relying_party.toml"
fi

# Generate relying party ephemeral ID secret
generate_ws_random_key ephemeral_id_secret
MRP_VERIFICATION_SERVER_EPHEMERAL_ID_SECRET=$(< "${TARGET_DIR}/mock_relying_party/ephemeral_id_secret.key" xxd -p | tr -d '\n')
export MRP_VERIFICATION_SERVER_EPHEMERAL_ID_SECRET

# And the mrp's wallet_server config
render_template "${DEVENV}/mrp_verification_server.toml.template" "${WALLET_SERVER_DIR}/verification_server.toml"

render_template "${DEVENV}/mrp_verification_server.toml.template" "${WALLET_SERVER_DIR}/ws_integration_test.toml"
render_template_append "${DEVENV}/mrp_verification_server.it.toml.template" "${WALLET_SERVER_DIR}/ws_integration_test.toml"

render_template "${DEVENV}/mrp_verification_server.toml.template" "${BASE_DIR}/wallet_core/tests_integration/wallet_server.toml"
render_template_append "${DEVENV}/mrp_verification_server.it.toml.template" "${BASE_DIR}/wallet_core/tests_integration/wallet_server.toml"

# And the pid_issuer config, for integration tests append to `verification_server.toml`
render_template "${DEVENV}/pid_issuer.toml.template" "${WALLET_SERVER_DIR}/pid_issuer.toml"

render_template "${DEVENV}/performance_test.env" "${BASE_DIR}/wallet_core/tests_integration/.env"

# And the wallet_server.toml, for integration tests
render_template "${DEVENV}/mrp_verification_server.toml.template" "${WALLET_SERVER_DIR}/wallet_server.toml"
render_template_append "${DEVENV}/pid_issuer.toml.template" "${WALLET_SERVER_DIR}/wallet_server.toml.in"
awk '/\[issuer/{p++}p' "${WALLET_SERVER_DIR}/wallet_server.toml.in" >> "${WALLET_SERVER_DIR}/wallet_server.toml"
rm "${WALLET_SERVER_DIR}/wallet_server.toml.in"

########################################################################
# Configure wallet_provider

echo
echo -e "${SECTION}Configure wallet_provider${NC}"

generate_wp_signing_key certificate_signing
WP_CERTIFICATE_SIGNING_KEY_PATH="${TARGET_DIR}/wallet_provider/certificate_signing.pem"
export WP_CERTIFICATE_SIGNING_KEY_PATH
WP_CERTIFICATE_PUBLIC_KEY=$(< "${TARGET_DIR}/wallet_provider/certificate_signing.pub.der" ${BASE64})
export WP_CERTIFICATE_PUBLIC_KEY

generate_wp_signing_key instruction_result_signing
WP_INSTRUCTION_RESULT_SIGNING_KEY_PATH="${TARGET_DIR}/wallet_provider/instruction_result_signing.pem"
export WP_INSTRUCTION_RESULT_SIGNING_KEY_PATH
WP_INSTRUCTION_RESULT_PUBLIC_KEY=$(< "${TARGET_DIR}/wallet_provider/instruction_result_signing.pub.der" ${BASE64})
export WP_INSTRUCTION_RESULT_PUBLIC_KEY

generate_wp_random_key attestation_wrapping
WP_ATTESTATION_WRAPPING_KEY_PATH="${TARGET_DIR}/wallet_provider/attestation_wrapping.key"
export WP_ATTESTATION_WRAPPING_KEY_PATH

generate_wp_random_key pin_pubkey_encryption
WP_PIN_PUBKEY_ENCRYPTION_KEY_PATH="${TARGET_DIR}/wallet_provider/pin_pubkey_encryption.key"
export WP_PIN_PUBKEY_ENCRYPTION_KEY_PATH

render_template "${DEVENV}/wallet_provider.toml.template" "${WP_DIR}/wallet_provider.toml"
render_template "${DEVENV}/wallet_provider.toml.template" "${BASE_DIR}/wallet_core/tests_integration/wallet_provider.toml"

render_template "${DEVENV}/wallet-config.json.template" "${TARGET_DIR}/wallet-config.json"

########################################################################
# Configure HSM

echo
echo -e "${SECTION}Configure HSM${NC}"

mkdir -p "${HOME}/.config/softhsm2"
if [ "${HSM_TOKEN_DIR}" = "${DEFAULT_HSM_TOKEN_DIR}" ]; then
  mkdir -p "${DEFAULT_HSM_TOKEN_DIR}"
fi

render_template "${DEVENV}/softhsm2/softhsm2.conf.template" "${HOME}/.config/softhsm2/softhsm2.conf"

softhsm2-util --delete-token --token test_token --force > /dev/null || true
softhsm2-util --init-token --slot 0 --so-pin "${HSM_SO_PIN}" --label "test_token" --pin "${HSM_USER_PIN}"
softhsm2-util --import "${WP_CERTIFICATE_SIGNING_KEY_PATH}" --pin "${HSM_USER_PIN}" --id "$(echo -n "certificate_signing" | xxd -p)" --label "certificate_signing_key" --token "test_token"
softhsm2-util --import "${WP_INSTRUCTION_RESULT_SIGNING_KEY_PATH}" --pin "${HSM_USER_PIN}" --id "$(echo -n "instruction_result_signing" | xxd -p)" --label "instruction_result_signing_key" --token "test_token"
softhsm2-util --import "${WP_ATTESTATION_WRAPPING_KEY_PATH}" --aes --pin "${HSM_USER_PIN}" --id "$(echo -n "attestation_wrapping" | xxd -p)" --label "attestation_wrapping_key" --token "test_token"
softhsm2-util --import "${WP_PIN_PUBKEY_ENCRYPTION_KEY_PATH}" --aes --pin "${HSM_USER_PIN}" --id "$(echo -n "pin_pubkey_encryption" | xxd -p)" --label "pin_pubkey_encryption_key" --token "test_token"

p11tool --login --write \
  --secret-key="$(openssl rand 32 | od -A n -v -t x1 | tr -d ' \n')" \
  --set-pin "${HSM_USER_PIN}" \
  --label="pin_public_disclosure_protection_key" \
  --provider="${HSM_LIBRARY_PATH}" \
  "$(p11tool --list-token-urls --provider="${HSM_LIBRARY_PATH}" | grep "SoftHSM")"


########################################################################
# Configure configuration-server

echo
echo -e "${SECTION}Configure configuration-server${NC}"

cd "${BASE_DIR}"

# Generate root CA
if [ ! -f "${TARGET_DIR}/configuration_server/ca.key.pem" ]; then
    generate_root_ca "${TARGET_DIR}/configuration_server" "nl-wallet-configuration-server"
else
    echo -e "${INFO}Target file '${TARGET_DIR}/configuration_server/ca.key.pem' already exists, not (re-)generating root CA"
fi

generate_ssl_key_pair_with_san "${TARGET_DIR}/configuration_server" config_server "${TARGET_DIR}/configuration_server/ca.crt.pem" "${TARGET_DIR}/configuration_server/ca.key.pem"

cp "${TARGET_DIR}/configuration_server/ca.crt.pem" "${BASE_DIR}/wallet_core/tests_integration/"
CONFIG_SERVER_CA_CRT=$(< "${TARGET_DIR}/configuration_server/ca.crt.der" ${BASE64})
export CONFIG_SERVER_CA_CRT

CONFIG_SERVER_CERT=$(< "${TARGET_DIR}/configuration_server/config_server.crt.der" ${BASE64})
export CONFIG_SERVER_CERT

CONFIG_SERVER_KEY=$(< "${TARGET_DIR}/configuration_server/config_server.key.der" ${BASE64})
export CONFIG_SERVER_KEY


generate_wp_signing_key config_signing
CONFIG_SIGNING_PUBLIC_KEY=$(< "${TARGET_DIR}/wallet_provider/config_signing.pub.der" ${BASE64})
export CONFIG_SIGNING_PUBLIC_KEY

BASE64_JWS_HEADER=$(echo -n '{"typ":"JOSE+JSON","alg":"ES256"}' | base64_url_encode)
BASE64_JWS_PAYLOAD=$(jq --compact-output --join-output "." "${TARGET_DIR}/wallet-config.json" | base64_url_encode)
BASE64_JWS_SIGNING_INPUT="${BASE64_JWS_HEADER}.${BASE64_JWS_PAYLOAD}"
DER_SIGNATURE=$(echo -n "$BASE64_JWS_SIGNING_INPUT" \
  | openssl dgst -sha256 -sign "${TARGET_DIR}/wallet_provider/config_signing.pem" -keyform PEM -binary \
  | openssl asn1parse -inform DER)
R=$(echo -n "${DER_SIGNATURE}" | grep 'INTEGER' | ${GNUSED} -n '1s/.*: //p' | ${GNUSED} -e 's/^INTEGER[[:space:]]*:\([[:alnum:]]*\)/\1/g')
S=$(echo -n "${DER_SIGNATURE}" | grep 'INTEGER' | ${GNUSED} -n '2s/.*: //p' | ${GNUSED} -e 's/^INTEGER[[:space:]]*:\([[:alnum:]]*\)/\1/g')
BASE64_JWS_SIGNATURE=$(echo -n "${R}${S}" | xxd -p -r | base64_url_encode)

echo -n "${BASE64_JWS_HEADER}.${BASE64_JWS_PAYLOAD}.${BASE64_JWS_SIGNATURE}" > "${TARGET_DIR}/wallet-config-jws-compact.txt"
cp "${TARGET_DIR}/wallet-config.json" "${BASE_DIR}/wallet_core/tests_integration/wallet-config.json"
cp "${TARGET_DIR}/wallet_provider/config_signing.pem" "${BASE_DIR}/wallet_core/tests_integration/config_signing.pem"


WALLET_CONFIG_JWT=$(< "${TARGET_DIR}/wallet-config-jws-compact.txt")
export WALLET_CONFIG_JWT

render_template "${DEVENV}/config_server.toml.template" "${CS_DIR}/config_server.toml"
cp "${CS_DIR}/config_server.toml" "${BASE_DIR}/wallet_core/tests_integration/config_server.toml"


########################################################################
# Configure gba-hc-converter

echo
echo -e "${SECTION}Configure gba-hc-converter${NC}"

render_template "${DEVENV}/gba_hc_converter.toml.template" "${BASE_DIR}/wallet_core/gba_hc_converter/gba_hc_converter.toml"
render_template "${DEVENV}/gba_hc_converter.toml.template" "${BASE_DIR}/wallet_core/tests_integration/gba_hc_converter.toml"

rm -rf "${GBA_HC_CONVERTER_DIR}/resources/encrypted-gba-v-responses"
encrypt_gba_v_responses

########################################################################
# Configure wallet

echo
echo -e "${SECTION}Configure wallet${NC}"

render_template "${DEVENV}/wallet.env.template" "${BASE_DIR}/wallet_core/wallet/.env"

########################################################################
# Configure Android Emulator

if command -v adb > /dev/null
then
    echo
    echo -e "${SECTION}Configure Android Emulator${NC}"

    "${SCRIPTS_DIR}"/map_android_ports.sh
fi

########################################################################
# Done

echo
echo -e "${SUCCESS}Setup complete${NC}"
