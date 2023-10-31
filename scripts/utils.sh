#!/usr/bin/env bash

########################################################################
# Colors
########################################################################

source "${SCRIPTS_DIR}/colors.sh"

SECTION=${LIGHT_BLUE}
SUCCESS=${LIGHT_GREEN}
INFO=${PURPLE}

########################################################################
# Functions
########################################################################

function is_macos() {
  uname -a | grep -i darwin >/dev/null
}

function detect_softhsm() {
  if is_macos; then
    find "${HOMEBREW_CELLAR}"/softhsm -name "libsofthsm2.so" | head -n 1
  else
    find "/usr/lib/" -name "libsofthsm2.so" | head -n 1
  fi
}

function check_openssl() {
  if ! openssl version | grep "OpenSSL" > /dev/null
  then
    echo -e "${RED}ERROR${NC}: Please install an OpenSSL version"
    exit 1
  fi
}

# Check whether COMMAND exists, and if not echo an error MESSAGE, and exit
#
# $1 - COMMAND: Name of the shell command
# $2 - MESSAGE: Error message to show when COMMAND does not exist
function expect_command {
    if ! command -v "$1" > /dev/null
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
    envsubst < "$1" > "$2"
}

# Return the body of a pem file.
# NOTE: This function will only work reliably on PEM files that contain a single object.
#
# $1 FILENAME of pem file
function get_pem_body {
    grep -v "\-\-\-\-\-" < "$1" | tr -d "\n"
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
            -pubout > "$1/$2.pub"
}

# Generate a private EC key and return the PEM body
#
# $1 name of the key
function generate_wp_signing_key {
    echo -e "${INFO}Generating EC private key${NC}"
    openssl ecparam \
            -genkey \
            -name prime256v1 \
            -noout \
            -out "${TARGET_DIR}/wallet_provider/$1.ec.key" > /dev/null
    echo -e "${INFO}Generating private key from EC private key${NC}"
    openssl ec -in "${TARGET_DIR}/wallet_provider/$1.ec.key" -pubout -out "${TARGET_DIR}/wallet_provider/$1.pub.pem"
    openssl pkey -in "${TARGET_DIR}/wallet_provider/$1.pub.pem" -pubin -outform der -out "${TARGET_DIR}/wallet_provider/$1.pub.der"

    openssl pkcs8 \
            -topk8 \
            -nocrypt \
            -in "${TARGET_DIR}/wallet_provider/$1.ec.key" \
            -out "${TARGET_DIR}/wallet_provider/$1.pem" > /dev/null
}

# Generate a symmetric key (AES-256)
#
# $1 name of the key
function generate_wp_symmetric_key {
    echo -e "${INFO}Generating symmetric key${NC}"
    openssl rand -out "${TARGET_DIR}/wallet_provider/$1.key" 32 > /dev/null
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
