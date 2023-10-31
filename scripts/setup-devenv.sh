#!/usr/bin/env bash
# This script generates configuration files, keys and certificates for a local development environment for the Wallet.
# It will configure the following applications:
#
# - nl-rdo-max-private (digid-connector)
#   This script requires this repo to exist in the same directory that contains the NL Wallet repo. Otherwise, customize
#   the DIGID_CONNECTOR_PATH environment variable in `scripts/.env`
# - pid_issuer
# - wallet_provider
# - wallet
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
#     adb reverse tcp:3000 tcp:3000
#     adb reverse tcp:3003 tcp:3003
#     adb reverse tcp:8006 tcp:8006
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
expect_command docker "Missing binary 'docker', please install Docker (Desktop)"
expect_command softhsm2-util "Missing binary 'softhsm2-util', please install softhsm2"
check_openssl

if is_macos
then
    expect_command gsed "Missing binary 'gsed', please install gnu-sed"
    GNUSED="gsed"
else
    GNUSED="sed"
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
mkdir -p "${TARGET_DIR}/pid_issuer"
mkdir -p "${TARGET_DIR}/wallet_provider"

########################################################################
# Configure digid-connector

echo
echo -e "${SECTION}Configure and start digid-connector${NC}"

cd "${DIGID_CONNECTOR_PATH}"
make setup

render_template "${DEVENV}/digid-connector/max.conf" "${DIGID_CONNECTOR_PATH}/max.conf"
render_template "${DEVENV}/digid-connector/clients.json" "${DIGID_CONNECTOR_PATH}/clients.json"

generate_ssl_key_pair_with_san "${DIGID_CONNECTOR_PATH}/secrets/ssl" server

# Build max docker container
docker compose build max
# Generate JWK from private RSA key of test_client.
CLIENT_PRIVKEY_JWK=$(docker compose run max make --silent create-jwk)
# Remove the 'kid' json field, otherwise the digid-connector will fail on it.
# TODO: find out what a correct value of 'kid' is and how to configure it for both the pid_issuer and digid-connector.
BSN_PRIVKEY=$(echo "${CLIENT_PRIVKEY_JWK}" | jq -c 'del(.kid)')
export BSN_PRIVKEY

########################################################################
# Configure pid_issuer

echo
echo -e "${SECTION}Configure pid_issuer${NC}"

cd "${BASE_DIR}"

# Generate root CA for cose signing
generate_pid_issuer_root_ca

# Generate pid issuer key and cert
generate_pid_issuer_key_pair

PID_CA_CRT=$(get_pem_body "${TARGET_DIR}/pid_issuer/ca_cert.pem")
export PID_CA_CRT
PID_ISSUER_KEY=$(get_pem_body "${TARGET_DIR}/pid_issuer/issuer_key.pem")
export PID_ISSUER_KEY
PID_ISSUER_CRT=$(get_pem_body "${TARGET_DIR}/pid_issuer/issuer_crt.pem")
export PID_ISSUER_CRT

render_template "${DEVENV}/pid_issuer.toml.template" "${PID_ISSUER_DIR}/pid_issuer.toml"
render_template "${DEVENV}/pid_issuer.toml.template" "${BASE_DIR}/wallet_core/tests_integration/pid_issuer.toml"

########################################################################
# Configure wallet_provider

echo
echo -e "${SECTION}Configure wallet_provider${NC}"

generate_wp_signing_key certificate_signing
WP_CERTIFICATE_SIGNING_KEY_PATH="${TARGET_DIR}/wallet_provider/certificate_signing.pem"
export WP_CERTIFICATE_SIGNING_KEY_PATH
WP_CERTIFICATE_PUBLIC_KEY=$(< "${TARGET_DIR}/wallet_provider/certificate_signing.pub.der" base64 | tr -d '\n')
export WP_CERTIFICATE_PUBLIC_KEY

generate_wp_signing_key instruction_result_signing
WP_INSTRUCTION_RESULT_SIGNING_KEY_PATH="${TARGET_DIR}/wallet_provider/instruction_result_signing.pem"
export WP_INSTRUCTION_RESULT_SIGNING_KEY_PATH
WP_INSTRUCTION_RESULT_PUBLIC_KEY=$(< "${TARGET_DIR}/wallet_provider/instruction_result_signing.pub.der" base64 | tr -d '\n')
export WP_INSTRUCTION_RESULT_PUBLIC_KEY

generate_wp_symmetric_key attestation_wrapping
WP_ATTESTATION_WRAPPING_KEY_PATH="${TARGET_DIR}/wallet_provider/attestation_wrapping.key"
export WP_ATTESTATION_WRAPPING_KEY_PATH

WP_PIN_HASH_SALT=$(openssl rand 32 | base64 | tr -d '=')
export WP_PIN_HASH_SALT

render_template "${DEVENV}/wallet_provider.toml.template" "${WP_DIR}/wallet_provider.toml"
render_template "${DEVENV}/wallet_provider.toml.template" "${BASE_DIR}/wallet_core/tests_integration/wallet_provider.toml"

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
# id = echo "certificate_signing" | xxd -p
softhsm2-util --import "${WP_CERTIFICATE_SIGNING_KEY_PATH}" --pin "${HSM_USER_PIN}" --id "63657274696669636174655f7369676e696e670a" --label "certificate_signing_key" --token "test_token"
# id = echo "instruction_result_signing" | xxd -p
softhsm2-util --import "${WP_INSTRUCTION_RESULT_SIGNING_KEY_PATH}" --pin "${HSM_USER_PIN}" --id "696e737472756374696f6e5f726573756c745f7369676e696e670a" --label "instruction_result_signing_key" --token "test_token"
# id = echo "attestation_wrapping" | xxd -p
softhsm2-util --import "${WP_ATTESTATION_WRAPPING_KEY_PATH}" --aes --pin "${HSM_USER_PIN}" --id "6174746573746174696f6e5f7772617070696e670a" --label "attestation_wrapping_key" --token "test_token"

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
