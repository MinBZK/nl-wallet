#!/usr/bin/env bash
# This script generates configuration files, keys and certificates for a local development environment for the Wallet.
# It will configure the following applications:
#
# - nl-rdo-max (digid-connector)
# - wallet_provider
# - wallet
# - pid_issuer, verification_server and issuance_server
# - demo_index, demo_issuer and demo_relying_party
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
# - envsubst: Environment variables substitution
# - make: used for building nl-rdo-max (digid-connector)
#
# MacOS specific instructions
# This script needs GNU sed. this can be installed by
#
#     brew install gnu-sed
#

set -e # break on error
set -u # warn against undefined variables
set -o pipefail
# set -x # echo statements before executing, useful while debugging

########################################################################
# Globals and includes
########################################################################

SCRIPTS_DIR="$(cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd)"
BASE_DIR=$(dirname "${SCRIPTS_DIR}")

source "${SCRIPTS_DIR}/utils.sh"

########################################################################
# Check prerequisites
########################################################################

# Use gsed on macOS, sed on others
is_macos && SED="gsed" || SED="sed"
have cargo jq tr xxd openssl p11tool softhsm2-util envsubst make "${SED}"

# Check if openssl is "real" OpenSSL
check_openssl

# Only check for docker if we build rdo-max
if [[ -z "${SKIP_DIGID_CONNECTOR:-}" ]]; then
    have docker
fi

# Only check for node if we build wallet_web
if [[ -z "${SKIP_WALLET_WEB:-}" ]]; then
    have node
fi

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
mkdir -p "${TARGET_DIR}/verification_server"
mkdir -p "${TARGET_DIR}/issuance_server"
mkdir -p "${TARGET_DIR}/demo_index"
mkdir -p "${TARGET_DIR}/demo_relying_party"
mkdir -p "${TARGET_DIR}/demo_issuer"
mkdir -p "${TARGET_DIR}/update_policy_server"
mkdir -p "${TARGET_DIR}/wallet_provider"

########################################################################
# Configure digid-connector
########################################################################

if [[ -z "${SKIP_DIGID_CONNECTOR:-}" ]]; then
  echo -e "${SECTION}Configure and start digid-connector${NC}"

  # Check for existing nl-rdo-max, re-use if existing, clone if not
  if [[ -d "${DIGID_CONNECTOR_PATH}" ]]; then
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
  DIGID_CA_CRT=$(< "${DIGID_CONNECTOR_PATH}/secrets/cacert.der" ${BASE64})
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
########################################################################

if [[ -z "${SKIP_WALLET_WEB:-}" ]]; then
    echo
    echo -e "${SECTION}Build wallet_web frontend${NC}"

    cd "${WALLET_WEB_DIR}"

    VITE_HELP_BASE_URL=${VITE_HELP_BASE_URL:-http://$SERVICES_HOST}
    export VITE_HELP_BASE_URL
    npm ci && npm run build

    cp dist/nl-wallet-web.iife.js ../wallet_core/demo/demo_utils/assets/

    # only do this locally
    cp dist/nl-wallet-web.iife.js ../wallet_core/demo/demo_relying_party/assets/
    cp dist/nl-wallet-web.iife.js ../wallet_core/demo/demo_issuer/assets/
fi


########################################################################
# Initialize HSM
########################################################################

echo
echo -e "${SECTION}Initialize HSM${NC}"

if [[ ! -f "$HSM_LIBRARY_PATH" ]]; then
    error "No valid HSM library path configured: $HSM_LIBRARY_PATH"
    exit 1
fi

mkdir -p "${HOME}/.config/softhsm2"
if [ "${HSM_TOKEN_DIR}" = "${DEFAULT_HSM_TOKEN_DIR}" ]; then
  mkdir -p "${DEFAULT_HSM_TOKEN_DIR}"
fi

render_template "${DEVENV}/softhsm2/softhsm2.conf.template" "${HOME}/.config/softhsm2/softhsm2.conf"

softhsm2-util --delete-token --token test_token --force > /dev/null || true
softhsm2-util --init-token --slot 0 --so-pin "${HSM_SO_PIN}" --label "test_token" --pin "${HSM_USER_PIN}"

render_template "${DEVENV}/hsm.toml.template" "${BASE_DIR}/wallet_core/lib/hsm/hsm.toml"

########################################################################
# Configure verification_server and demo_relying_party
########################################################################

echo
echo -e "${SECTION}Configure verification_server and demo_relying_party${NC}"

cd "${BASE_DIR}"

# Generate root CA for issuer
if [ ! -f "${TARGET_DIR}/ca.issuer.key.pem" ]; then
    generate_issuer_root_ca
else
    echo -e "${INFO}Target file '${TARGET_DIR}/ca.issuer.key.pem' already exists, not (re-)generating issuer root CA"
fi
ISSUER_CA_CRT=$(< "${TARGET_DIR}/ca.issuer.crt.der" ${BASE64})
export ISSUER_CA_CRT

# Generate key for WTE signing
generate_wp_signing_key wte_signing
WP_WTE_SIGNING_KEY_PATH="${TARGET_DIR}/wallet_provider/wte_signing.pem"
export WP_WTE_SIGNING_KEY_PATH
WP_WTE_PUBLIC_KEY=$(< "${TARGET_DIR}/wallet_provider/wte_signing.pub.der" ${BASE64})
export WP_WTE_PUBLIC_KEY

# Generate pid issuer key and cert
generate_pid_issuer_key_pair

PID_ISSUER_KEY=pid_issuer_key
export PID_ISSUER_KEY
PID_ISSUER_CRT=$(< "${TARGET_DIR}/pid_issuer/issuer.crt.der" ${BASE64})
export PID_ISSUER_CRT

# Generate root CA for reader
if [ ! -f "${TARGET_DIR}/ca.reader.key.pem" ]; then
    generate_reader_root_ca
else
    echo -e "${INFO}Target file '${TARGET_DIR}/ca.reader.key.pem' already exists, not (re-)generating reader root CA"
fi

# Generate CA for RPs
READER_CA_CRT=$(< "${TARGET_DIR}/ca.reader.crt.der" ${BASE64})
export READER_CA_CRT

# Generate relying party key and cert
generate_relying_party_hsm_key_pair mijn_amsterdam demo_relying_party
DEMO_RELYING_PARTY_KEY_MIJN_AMSTERDAM=mijn_amsterdam_key
export DEMO_RELYING_PARTY_KEY_MIJN_AMSTERDAM
DEMO_RELYING_PARTY_CRT_MIJN_AMSTERDAM=$(< "${TARGET_DIR}/demo_relying_party/mijn_amsterdam.crt.der" ${BASE64})
export DEMO_RELYING_PARTY_CRT_MIJN_AMSTERDAM

# Generate relying party key and cert
generate_demo_relying_party_key_pair online_marketplace
DEMO_RELYING_PARTY_KEY_ONLINE_MARKETPLACE=$(< "${TARGET_DIR}/demo_relying_party/online_marketplace.key.der" ${BASE64})
export DEMO_RELYING_PARTY_KEY_ONLINE_MARKETPLACE
DEMO_RELYING_PARTY_CRT_ONLINE_MARKETPLACE=$(< "${TARGET_DIR}/demo_relying_party/online_marketplace.crt.der" ${BASE64})
export DEMO_RELYING_PARTY_CRT_ONLINE_MARKETPLACE

# Generate relying party key and cert
generate_demo_relying_party_key_pair xyz_bank
DEMO_RELYING_PARTY_KEY_XYZ_BANK=$(< "${TARGET_DIR}/demo_relying_party/xyz_bank.key.der" ${BASE64})
export DEMO_RELYING_PARTY_KEY_XYZ_BANK
DEMO_RELYING_PARTY_CRT_XYZ_BANK=$(< "${TARGET_DIR}/demo_relying_party/xyz_bank.crt.der" ${BASE64})
export DEMO_RELYING_PARTY_CRT_XYZ_BANK

# Generate relying party key and cert
generate_demo_relying_party_key_pair monkey_bike
DEMO_RELYING_PARTY_KEY_MONKEY_BIKE=$(< "${TARGET_DIR}/demo_relying_party/monkey_bike.key.der" ${BASE64})
export DEMO_RELYING_PARTY_KEY_MONKEY_BIKE
DEMO_RELYING_PARTY_CRT_MONKEY_BIKE=$(< "${TARGET_DIR}/demo_relying_party/monkey_bike.crt.der" ${BASE64})
export DEMO_RELYING_PARTY_CRT_MONKEY_BIKE

# Generate relying party key and cert
generate_demo_relying_party_key_pair job_finder
DEMO_RELYING_PARTY_KEY_JOB_FINDER=$(< "${TARGET_DIR}/demo_relying_party/job_finder.key.der" ${BASE64})
export DEMO_RELYING_PARTY_KEY_JOB_FINDER
DEMO_RELYING_PARTY_CRT_JOB_FINDER=$(< "${TARGET_DIR}/demo_relying_party/job_finder.crt.der" ${BASE64})
export DEMO_RELYING_PARTY_CRT_JOB_FINDER

render_template "${DEVENV}/demo_relying_party.toml.template" "${DEMO_RELYING_PARTY_DIR}/demo_relying_party.toml"


# Generate issuer key and cert
generate_demo_issuer_key_pairs university
DEMO_ISSUER_KEY_UNIVERSITY_READER=$(< "${TARGET_DIR}/demo_issuer/university.reader.key.der" ${BASE64})
export DEMO_ISSUER_KEY_UNIVERSITY_READER
DEMO_ISSUER_CRT_UNIVERSITY_READER=$(< "${TARGET_DIR}/demo_issuer/university.reader.crt.der" ${BASE64})
export DEMO_ISSUER_CRT_UNIVERSITY_READER
DEMO_ISSUER_KEY_UNIVERSITY_ISSUER=$(< "${TARGET_DIR}/demo_issuer/university.issuer.key.der" ${BASE64})
export DEMO_ISSUER_KEY_UNIVERSITY_ISSUER
DEMO_ISSUER_CRT_UNIVERSITY_ISSUER=$(< "${TARGET_DIR}/demo_issuer/university.issuer.crt.der" ${BASE64})
export DEMO_ISSUER_CRT_UNIVERSITY_ISSUER

generate_demo_issuer_key_pairs insurance
DEMO_ISSUER_KEY_INSURANCE_READER=$(< "${TARGET_DIR}/demo_issuer/insurance.reader.key.der" ${BASE64})
export DEMO_ISSUER_KEY_INSURANCE_READER
DEMO_ISSUER_CRT_INSURANCE_READER=$(< "${TARGET_DIR}/demo_issuer/insurance.reader.crt.der" ${BASE64})
export DEMO_ISSUER_CRT_INSURANCE_READER
DEMO_ISSUER_KEY_INSURANCE_ISSUER=$(< "${TARGET_DIR}/demo_issuer/insurance.issuer.key.der" ${BASE64})
export DEMO_ISSUER_KEY_INSURANCE_ISSUER
DEMO_ISSUER_CRT_INSURANCE_ISSUER=$(< "${TARGET_DIR}/demo_issuer/insurance.issuer.crt.der" ${BASE64})
export DEMO_ISSUER_CRT_INSURANCE_ISSUER

render_template "${DEVENV}/demo_issuer.toml.template" "${DEMO_ISSUER_DIR}/demo_issuer.toml"


# Generate relying party ephemeral ID secret
generate_ws_random_key ephemeral_id_secret
DEMO_RP_VERIFICATION_SERVER_EPHEMERAL_ID_SECRET=$(< "${TARGET_DIR}/demo_relying_party/ephemeral_id_secret.key" xxd -p | tr -d '\n')
export DEMO_RP_VERIFICATION_SERVER_EPHEMERAL_ID_SECRET

render_template "${DEVENV}/demo_index.toml.template" "${DEMO_INDEX_DIR}/demo_index.toml"

# Copy the Technical Attestation Schemas
cp "${DEVENV}/eudi:pid:1.json" "${DEVENV}/eudi:pid:nl:1.json" "${DEVENV}/eudi:pid-address:1.json" "${DEVENV}/eudi:pid-address:nl:1.json" "${PID_ISSUER_DIR}"
cp "${DEVENV}/eudi:pid:1.json" "${DEVENV}/eudi:pid:nl:1.json" "${DEVENV}/eudi:pid-address:1.json" "${DEVENV}/eudi:pid-address:nl:1.json" "${DEVENV}/com.example.degree.json" "${DEVENV}/com.example.insurance.json" "${BASE_DIR}/wallet_core/tests_integration"
cp "${DEVENV}/com.example.degree.json" "${DEVENV}/com.example.insurance.json" "${ISSUANCE_SERVER_DIR}"
ISSUER_METADATA_PID_PATH="eudi:pid:1.json"
export ISSUER_METADATA_PID_PATH
ISSUER_METADATA_PID_NL_PATH="eudi:pid:nl:1.json"
export ISSUER_METADATA_PID_NL_PATH
ISSUER_METADATA_ADDRESS_PATH="eudi:pid-address:1.json"
export ISSUER_METADATA_ADDRESS_PATH
ISSUER_METADATA_ADDRESS_NL_PATH="eudi:pid-address:nl:1.json"
export ISSUER_METADATA_ADDRESS_NL_PATH
ISSUER_METADATA_DEGREE_PATH="com.example.degree.json"
export ISSUER_METADATA_DEGREE_PATH
ISSUER_METADATA_INSURANCE_PATH="com.example.insurance.json"
export ISSUER_METADATA_INSURANCE_PATH

# And the demo RP's verification_server config
render_template "${DEVENV}/demo_rp_verification_server.toml.template" "${VERIFICATION_SERVER_DIR}/verification_server.toml"
render_template "${DEVENV}/demo_rp_verification_server.toml.template" "${BASE_DIR}/wallet_core/tests_integration/verification_server.toml"

# And the pid_issuer config
render_template "${DEVENV}/pid_issuer.toml.template" "${PID_ISSUER_DIR}/pid_issuer.toml"
render_template "${DEVENV}/pid_issuer.toml.template" "${BASE_DIR}/wallet_core/tests_integration/pid_issuer.toml"

# And the issuance_server config
render_template "${DEVENV}/demo_issuer_issuance_server.toml.template" "${ISSUANCE_SERVER_DIR}/issuance_server.toml"
render_template "${DEVENV}/demo_issuer_issuance_server.toml.template" "${BASE_DIR}/wallet_core/tests_integration/issuance_server.toml"

render_template "${DEVENV}/performance_test.env" "${BASE_DIR}/wallet_core/tests_integration/.env"

########################################################################
# Configure update-policy-server

echo
echo -e "${SECTION}Configure update-policy-server${NC}"

cd "${BASE_DIR}"

# Generate root CA
if [ ! -f "${TARGET_DIR}/update_policy_server/ca.key.pem" ]; then
    generate_root_ca "${TARGET_DIR}/update_policy_server" "nl-wallet-update-policy-server"
else
    echo -e "${INFO}Target file '${TARGET_DIR}/update_policy_server/ca.key.pem' already exists, not (re-)generating root CA"
fi

generate_ssl_key_pair_with_san "${TARGET_DIR}/update_policy_server" update_policy_server "${TARGET_DIR}/update_policy_server/ca.crt.pem" "${TARGET_DIR}/update_policy_server/ca.key.pem"

cp "${TARGET_DIR}/update_policy_server/ca.crt.pem" "${BASE_DIR}/wallet_core/tests_integration/ups.ca.crt.pem"
cp "${TARGET_DIR}/update_policy_server/ca.crt.der" "${BASE_DIR}/wallet_core/tests_integration/ups.ca.crt.der"
UPDATE_POLICY_SERVER_CA_CRT=$(< "${TARGET_DIR}/update_policy_server/ca.crt.der" ${BASE64})
export UPDATE_POLICY_SERVER_CA_CRT

UPDATE_POLICY_SERVER_CERT=$(< "${TARGET_DIR}/update_policy_server/update_policy_server.crt.der" ${BASE64})
export UPDATE_POLICY_SERVER_CERT

UPDATE_POLICY_SERVER_KEY=$(< "${TARGET_DIR}/update_policy_server/update_policy_server.key.der" ${BASE64})
export UPDATE_POLICY_SERVER_KEY

UPDATE_POLICY_SERVER_TRUST_ANCHORS=$(IFS="|" ; echo "${UPDATE_POLICY_SERVER_CERT[*]}")
export UPDATE_POLICY_SERVER_TRUST_ANCHORS

render_template "${DEVENV}/update_policy_server.toml.template" "${UPS_DIR}/update_policy_server.toml"
cp "${UPS_DIR}/update_policy_server.toml" "${BASE_DIR}/wallet_core/tests_integration/update_policy_server.toml"

########################################################################
# Configure wallet_provider
########################################################################

echo
echo -e "${SECTION}Configure wallet_provider${NC}"


# Generate root CA
if [ ! -f "${TARGET_DIR}/wallet_provider/ca.key.pem" ]; then
    generate_root_ca "${TARGET_DIR}/wallet_provider" "nl-wallet-provider"
else
    echo -e "${INFO}Target file '${TARGET_DIR}/wallet_provider/ca.key.pem' already exists, not (re-)generating root CA"
fi

generate_ssl_key_pair_with_san "${TARGET_DIR}/wallet_provider" wallet_provider "${TARGET_DIR}/wallet_provider/ca.crt.pem" "${TARGET_DIR}/wallet_provider/ca.key.pem"

cp "${TARGET_DIR}/wallet_provider/ca.crt.der" "${BASE_DIR}/wallet_core/tests_integration/wp.ca.crt.der"
WALLET_PROVIDER_SERVER_CA_CRT=$(< "${TARGET_DIR}/wallet_provider/ca.crt.der" ${BASE64})
export WALLET_PROVIDER_SERVER_CA_CRT

WALLET_PROVIDER_SERVER_CERT=$(< "${TARGET_DIR}/wallet_provider/wallet_provider.crt.der" ${BASE64})
export WALLET_PROVIDER_SERVER_CERT

WALLET_PROVIDER_SERVER_KEY=$(< "${TARGET_DIR}/wallet_provider/wallet_provider.key.der" ${BASE64})
export WALLET_PROVIDER_SERVER_KEY


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

APPLE_ROOT_CA=$(openssl x509 -in "${SCRIPTS_DIR}/../wallet_core/lib/apple_app_attest/assets/Apple_App_Attestation_Root_CA.pem" -outform DER | ${BASE64})
export APPLE_ROOT_CA

MOCK_APPLE_ROOT_CA=$(openssl x509 -in "${SCRIPTS_DIR}/../wallet_core/lib/apple_app_attest/assets/mock_ca.crt.pem" -outform DER | ${BASE64})
export MOCK_APPLE_ROOT_CA

# Source: https://developer.android.com/privacy-and-security/security-key-attestation#root_certificate
ANDROID_ROOT_PUBKEY=$(openssl rsa -pubin -in "${SCRIPTS_DIR}/../wallet_core/lib/android_attest/assets/google_hardware_attestation_root_pubkey.pem" -outform DER | ${BASE64})
export ANDROID_ROOT_PUBKEY

# Source: repository https://android.googlesource.com/platform/hardware/interfaces, file security/keymint/aidl/default/ta/attest.rs, variable EC_ATTEST_ROOT_CERT
ANDROID_EMULATOR_EC_ROOT_PUBKEY=$(openssl ec -pubin -in "${SCRIPTS_DIR}/../wallet_core/lib/android_attest/assets/android_emulator_ec_root_pubkey.pem" -outform DER | ${BASE64})
export ANDROID_EMULATOR_EC_ROOT_PUBKEY

# Source: repository https://android.googlesource.com/platform/hardware/interfaces, file security/keymint/aidl/default/ta/attest.rs, variable RSA_ATTEST_ROOT_CERT
ANDROID_EMULATOR_RSA_ROOT_PUBKEY=$(openssl rsa -pubin -in "${SCRIPTS_DIR}/../wallet_core/lib/android_attest/assets/android_emulator_rsa_root_pubkey.pem" -outform DER | ${BASE64})
export ANDROID_EMULATOR_RSA_ROOT_PUBKEY

render_template "${DEVENV}/wallet_provider.toml.template" "${WP_DIR}/wallet_provider.toml"
render_template "${DEVENV}/wallet_provider.toml.template" "${BASE_DIR}/wallet_core/tests_integration/wallet_provider.toml"

render_template "${DEVENV}/wallet-config.json.template" "${TARGET_DIR}/wallet-config.json"

########################################################################
# Configure HSM
########################################################################

softhsm2-util --import "${WP_CERTIFICATE_SIGNING_KEY_PATH}" --pin "${HSM_USER_PIN}" --id "$(echo -n "certificate_signing" | xxd -p)" --label "certificate_signing_key" --token "test_token"
softhsm2-util --import "${WP_WTE_SIGNING_KEY_PATH}" --pin "${HSM_USER_PIN}" --id "$(echo -n "wte_signing" | xxd -p)" --label "wte_signing_key" --token "test_token"
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
########################################################################

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

cp "${TARGET_DIR}/configuration_server/ca.crt.der" "${BASE_DIR}/wallet_core/tests_integration/cs.ca.crt.der"
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
RS=$(echo "${DER_SIGNATURE}" | awk -v len=64 'BEGIN { FS=":"; ORS=""; } /INTEGER/ { printf "%0" len "s", $4; }')
BASE64_JWS_SIGNATURE=$(echo -n "${RS}" | xxd -p -r | base64_url_encode)

echo -n "${BASE64_JWS_HEADER}.${BASE64_JWS_PAYLOAD}.${BASE64_JWS_SIGNATURE}" > "${TARGET_DIR}/wallet-config-jws-compact.txt"
cp "${TARGET_DIR}/wallet-config.json" "${BASE_DIR}/wallet_core/tests_integration/wallet-config.json"
cp "${TARGET_DIR}/wallet_provider/config_signing.pem" "${BASE_DIR}/wallet_core/tests_integration/config_signing.pem"


WALLET_CONFIG_JWT=$(< "${TARGET_DIR}/wallet-config-jws-compact.txt")
export WALLET_CONFIG_JWT
render_template "${DEVENV}/config_server.toml.template" "${CS_DIR}/config_server.toml"
cp "${CS_DIR}/config_server.toml" "${BASE_DIR}/wallet_core/tests_integration/config_server.toml"

########################################################################
# Configure gba-hc-converter
########################################################################

echo
echo -e "${SECTION}Configure gba-hc-converter${NC}"

render_template "${DEVENV}/gba_hc_converter.toml.template" "${BASE_DIR}/wallet_core/gba_hc_converter/gba_hc_converter.toml"
render_template "${DEVENV}/gba_hc_converter.toml.template" "${BASE_DIR}/wallet_core/tests_integration/gba_hc_converter.toml"

rm -rf "${GBA_HC_CONVERTER_DIR}/resources/encrypted-gba-v-responses"
encrypt_gba_v_responses

########################################################################
# Configure wallet
########################################################################

echo
echo -e "${SECTION}Configure wallet${NC}"

rm -f "${BASE_DIR}/wallet_core/wallet/config-server-config.json" "${BASE_DIR}/wallet_core/wallet/wallet-config.json"
render_template "${DEVENV}/config-server-config.json.template" "${BASE_DIR}/wallet_core/wallet/config-server-config.json"
render_template "${DEVENV}/wallet-config.json.template" "${BASE_DIR}/wallet_core/wallet/wallet-config.json"

########################################################################
# Configure Android Emulator
########################################################################

if command -v adb > /dev/null
then
    echo
    echo -e "${SECTION}Configure Android Emulator${NC}"

    "${SCRIPTS_DIR}"/map_android_ports.sh
fi

########################################################################
# Done
########################################################################

echo
echo -e "${SUCCESS}Setup complete${NC}"
