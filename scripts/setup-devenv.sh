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
#
# User specific variables can be supplied in the `.env` files.
#
# Prerequisites:
#
# - cargo: needed to build/run wallet_provider_configuration
# - openssl: needed to generate keys and certificates
# - jq: needed to parse JSON
# - standard unix tools like: grep, sed, tr, ...
# - docker: with compose extension, to run the digid-connector
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
# set -x # echo statements before executing, usefull while debugging

########################################################################
# Configuration
########################################################################

export SCRIPTS_DIR=$(dirname $(realpath $(command -v ${BASH_SOURCE[0]})))
export BASE_DIR=$(dirname $SCRIPTS_DIR)

source ${SCRIPTS_DIR}/configuration.sh

if [ ! -f "${SCRIPTS_DIR}/.env" ]
then
    echo -e "${INFO}Saving initial environment variables${NC}"
    echo -e \
"#!/usr/bin/env bash
# export DIGID_CONNECTOR_PATH=${DIGID_CONNECTOR_PATH}
# export DB_HOST=${DB_HOST}
# export DB_USERNAME=${DB_USERNAME}
# export DB_PASSWORD=${DB_PASSWORD}
# export DB_NAME=${DB_NAME}" \
    > "${SCRIPTS_DIR}/.env"
    echo -e "${INFO}Your customizations are saved in ${CYAN}${SCRIPTS_DIR}/.env${NC}"
    echo -e "${INFO}Edit this file to reflect your environment, and uncomment the relevant variables${NC}"
    echo -e "${INFO}Note that variables in this file can no longer be overridden by the commandline${NC}"
fi

########################################################################
# Functions
########################################################################

# Check whether COMMAND exists, and if not echo an error MESSAGE, and exit
#
# $1 - COMMAND: Name of the shell command
# $2 - MESSAGE: Error message to show when COMMAND does not exist
function expect_command {
    if ! command -v $1 > /dev/null
    then
        echo -e "${RED}ERROR${NC}: $2"
        exit 1
    fi
}

# Execute envsubst on TEMPLATE and write the result to TARGET
#
# $1 - TEMPLATE: Template file
# $2 - TARGET: Target file
function render_template {
    echo -e "Generating ${CYAN}$2${NC} from template ${CYAN}$1${NC}"
    cat "$1" | envsubst > "$2"
}

# Return the body of a pem file.
# NOTE: This function will only work reliably on PEM files that contain a single object.
#
# $1 FILENAME of pem file
function get_pem_body {
    cat "$1" | grep -v "\-\-\-\-\-" | tr -d "\n"
}

# Generate a key and certificate to use for local TLS.
# The generated certificate will have the following SAN names:
#
# [alt_names]
# IP.1    = 10.0.2.2 # special IP address for android emulator
# DNS.1   = localhost
#
# The certificate will be signed by the cacert of the digid-connector.
#
# $1 - TARGET: Target directory
# $2 - NAME: Key name, used as file name without extension
function generate_ssl_key_pair_with_san {
    echo -e "${INFO}Generating SSL private key and CSR${NC}"
    openssl req \
            -newkey rsa:2048 \
            -subj "/C=US/CN=localhost" \
            -nodes \
            -sha256 \
            -keyout "$1/$2.key" \
            -out "$1/$2.csr" \
            -config "${DEVENV}/openssl-san.cfg" 2>/dev/null
    echo -e "${INFO}Generating SSL CERT from CSR${NC}"
    openssl x509 \
            -req \
            -sha256 \
            -CAcreateserial \
            -in "$1/$2.csr" \
            -days 500 \
            -CA "${DIGID_CONNECTOR_PATH}/secrets/cacert.crt" \
            -CAkey "${DIGID_CONNECTOR_PATH}/secrets/cacert.key" \
            -out "$1/$2.crt" \
            -extensions req_ext \
            -extfile "${DEVENV}/openssl-san.cfg" 2>/dev/null
    echo -e "${INFO}Exporting SSL public key${NC}"
    openssl rsa \
            -in "$1/$2.key" \
            -pubout > "$1/$2.pub" > /dev/null
}

# Generate a private EC key and return the PEM body
#
# $1 name of the key
function generate_wp_private_key {
    echo -e "${INFO}Generating EC private key${NC}"
    openssl ecparam \
            -genkey \
            -name prime256v1 \
            -noout \
            -out "${TARGET_DIR}/wallet_provider/$1.ec.key" > /dev/null
    echo -e "${INFO}Generating private key from EC private key${NC}"
    openssl pkcs8 \
            -topk8 \
            -nocrypt \
            -in "${TARGET_DIR}/wallet_provider/$1.ec.key" \
            -out "${TARGET_DIR}/wallet_provider/$1.pem" > /dev/null
}

# Generate an EC root CA for the pid_issuer
function generate_pid_issuer_root_ca {
    echo -e "${INFO}Generating EC root CA${NC}"
    openssl req -x509 \
            -nodes \
            -newkey ec \
            -pkeyopt ec_paramgen_curve:prime256v1 \
            -keyout "${TARGET_DIR}/pid_issuer/ca_privkey.pem" \
            -out "${TARGET_DIR}/pid_issuer/ca_cert.pem" \
            -days 365 \
            -addext keyUsage=keyCertSign,cRLSign \
            -subj '/CN=ca.example.com'
}

# Generate an EC key pair for the pid_issuer
function generate_pid_issuer_key_pair {
    echo -e "${INFO}Generating EC issuer private key and CSR${NC}"
    openssl req -new \
            -nodes \
            -newkey ec \
            -pkeyopt ec_paramgen_curve:prime256v1 \
            -keyout "${TARGET_DIR}/pid_issuer/issuer_key.pem" \
            -out "${TARGET_DIR}/pid_issuer/issuer_csr.pem" \
            -subj "/CN=pid.example.com"
    echo -e "${INFO}Generate EC certificate from CSR using EC root CA${NC}"
    openssl x509 -req \
            -extfile <(printf "keyUsage=digitalSignature\nextendedKeyUsage=1.0.18013.5.1.2\nbasicConstraints=CA:FALSE") \
            -in "${TARGET_DIR}/pid_issuer/issuer_csr.pem" \
            -days 500 \
            -CA "${TARGET_DIR}/pid_issuer/ca_cert.pem" \
            -CAkey "${TARGET_DIR}/pid_issuer/ca_privkey.pem" \
            -out "${TARGET_DIR}/pid_issuer/issuer_crt.pem"
}

########################################################################
# Check prerequisites

expect_command cargo "Missing binary 'cargo', please install the Rust toolchain"
expect_command openssl "Missing binary 'openssl', please install OpenSSL"
expect_command jq "Missing binary 'jq', please install"
expect_command docker "Missing binary 'docker', please install Docker (Desktop)"

if [ $(uname -s) == "Darwin" ]
then
    expect_command gsed "Missing binary 'gsed', please install gnu-sed"
    GNUSED=gsed
else
    GNUSED=sed
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
export BSN_PRIVKEY=$(echo ${CLIENT_PRIVKEY_JWK} | jq -c 'del(.kid)')

########################################################################
# Configure pid_issuer

echo
echo -e "${SECTION}Configure pid_issuer${NC}"

cd "${BASE_DIR}"

# Generate root CA for cose signing
generate_pid_issuer_root_ca

# Generate pid issuer key and cert
generate_pid_issuer_key_pair

export PID_CA_CRT=$(get_pem_body "${TARGET_DIR}/pid_issuer/ca_cert.pem")
export PID_ISSUER_KEY=$(get_pem_body "${TARGET_DIR}/pid_issuer/issuer_key.pem")
export PID_ISSUER_CRT=$(get_pem_body "${TARGET_DIR}/pid_issuer/issuer_crt.pem")

render_template "${DEVENV}/pid_issuer.toml.template" "${PID_ISSUER_DIR}/pid_issuer.toml"
render_template "${DEVENV}/pid_issuer.toml.template" "${BASE_DIR}/wallet_core/tests_integration/pid_issuer.toml"

########################################################################
# Configure wallet_provider

echo
echo -e "${SECTION}Configure wallet_provider${NC}"

generate_wp_private_key certificate
export WP_CERTIFICATE_KEY=$(get_pem_body "${TARGET_DIR}/wallet_provider/certificate.pem")

generate_wp_private_key instruction_result
export WP_INSTRUCTION_RESULT_KEY=$(get_pem_body "${TARGET_DIR}/wallet_provider/instruction_result.pem")

export WP_PIN_HASH_SALT=$(openssl rand 32 | base64 | tr -d '=')

render_template "${DEVENV}/wallet_provider.toml.template" "${WP_DIR}/wallet_provider.toml"
render_template "${DEVENV}/wallet_provider.toml.template" "${BASE_DIR}/wallet_core/tests_integration/wallet_provider.toml"

# Get wallet_provider verifying keys
cd "$BASE_DIR/wallet_core/wallet_provider"
echo -e "${INFO}Exporting wallet_provider verifying keys${NC}"
WALLET_PROVIDER_CONFIGURATION=$(cargo run --bin wallet_provider_configuration)

export WP_CERTIFICATE_PUBLIC_KEY=$(echo ${WALLET_PROVIDER_CONFIGURATION} | jq -r '.certificate_verifying_key')
export WP_INSTRUCTION_RESULT_PUBLIC_KEY=$(echo ${WALLET_PROVIDER_CONFIGURATION} | jq -r '.instruction_result_verifying_key')

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

    ${SCRIPTS_DIR}/map_android_ports.sh
fi

########################################################################
# Done

echo
echo -e "${SUCCESS}Setup complete${NC}"
