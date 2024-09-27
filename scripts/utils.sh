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
  # shellcheck disable=SC2206
  local locations=("/usr/local/lib" ${NIX_PROFILES:-} "${HOMEBREW_PREFIX:+${HOMEBREW_PREFIX}/lib}" "/usr/lib")

  for location in "${locations[@]}"; do
      local library_path
      if [ -n "$location" ]; then
        library_path=$(find -L "$location" -maxdepth 3 -name "libsofthsm2.so" | head -n 1)
        if [ -n "$library_path" ]; then
            echo "$library_path"
            return
        fi
      fi
  done
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

# Execute envsubst on TEMPLATE and append the result to TARGET
#
# $1 - TEMPLATE: Template file
# $2 - TARGET: Target file
function render_template_append {
    echo -e "Appending to ${CYAN}$2${NC} from template ${CYAN}$1${NC}"
    envsubst < "$1" >> "$2"
}

# Generate n random bytes.
#
# $1 n: how many random bytes to generate
function random_bytes {
    dd if=/dev/urandom bs="$1" count=1 2>/dev/null
}

# Generate a key and certificate to use as root CA.
#
# $1 - Target directory
# $2 - Common name
function generate_root_ca {
    openssl req -subj "/C=NL/CN=$2" -nodes -x509 -sha256 -days 1825 -newkey rsa:2048 -keyout "$1/ca.key.pem" -out "$1/ca.crt.pem" > /dev/null
    openssl pkcs8 -topk8 -inform PEM -outform DER -in "$1/ca.key.pem" -out "$1/ca.key.der" -nocrypt
    openssl x509 -in "$1/ca.crt.pem" -outform DER -out "$1/ca.crt.der"
}

# Generate a key and certificate to use for local TLS.
# The generated certificate will have the following SAN names:
#
# [alt_names]
# IP.1    = 10.0.2.2 # special IP address for android emulator
# DNS.1   = localhost
#
# $1 - TARGET: Target directory
# $2 - NAME: Key name, used as file name without extension
# $3 - CA PUBLIC KEY: CA certificate signing public key
# $4 - CA PRIVATE KEY: CA certificate signing private key
function generate_ssl_key_pair_with_san {
    echo -e "${INFO}Generating SSL private key and CSR${NC}"
    openssl req \
            -newkey rsa:2048 \
            -subj "/C=US/CN=localhost" \
            -nodes \
            -sha256 \
            -keyout "$1/$2.key" \
            -out "$1/$2.csr" \
            -config "${DEVENV}/openssl-san.cfg" > /dev/null

    echo -e "${INFO}Generating SSL CERT from CSR${NC}"
    openssl x509 \
            -req \
            -sha256 \
            -in "$1/$2.csr" \
            -days 500 \
            -CA "$3" \
            -CAkey "$4" \
            -out "$1/$2.crt" \
            -extensions req_ext \
            -extfile "${DEVENV}/openssl-san.cfg" > /dev/null

    echo -e "${INFO}Exporting SSL public key${NC}"
    openssl rsa \
            -in "$1/$2.key" \
            -pubout > "$1/$2.pub"

    echo -e "${INFO}Converting SSL CERT to DER${NC}"
    openssl x509 -in "$1/$2.crt" \
                -outform der -out "$1/$2.crt.der"

    echo -e "${INFO}Converting SSL private key to DER${NC}"
    openssl pkcs8 -topk8 -inform PEM -outform DER \
            -in "$1/$2.key" -out "$1/$2.key.der" -nocrypt
}

# $1 name of the key
function generate_ec_key {
    echo -e "${INFO}Generating EC private key${NC}"
    openssl ecparam \
            -genkey \
            -name prime256v1 \
            -noout \
            -out "${TARGET_DIR}/wallet_provider/$1.ec.key" > /dev/null
    echo -e "${INFO}Generating private key from EC private key${NC}"
    openssl ec -in "${TARGET_DIR}/wallet_provider/$1.ec.key" -pubout -out "${TARGET_DIR}/wallet_provider/$1.pub.pem"
    openssl pkey -in "${TARGET_DIR}/wallet_provider/$1.pub.pem" -pubin -outform der -out "${TARGET_DIR}/wallet_provider/$1.pub.der"
}

# $1 name of the key
function private_key_to_pem {
    openssl pkcs8 \
            -topk8 \
            -nocrypt \
            -in "${TARGET_DIR}/wallet_provider/$1.ec.key" \
            -out "${TARGET_DIR}/wallet_provider/$1.pem" > /dev/null
}

# Generate a private EC key and return the PEM body
#
# $1 name of the key
function generate_wp_signing_key {
    generate_ec_key "$1"

    private_key_to_pem "$1"
}

# Generate a private EC key and a self-signed certificate
#
# $1 name of the key
# $2 common name to use in the certificate
function generate_wp_self_signed_certificate {
    generate_ec_key "$1"

    openssl req \
            -new \
            -subj "/CN=$2" \
            -days 1825 \
            -x509 \
            -sha256 \
            -outform der \
            -key "${TARGET_DIR}/wallet_provider/$1.ec.key" \
            -out "${TARGET_DIR}/wallet_provider/$1.crt" > /dev/null

    private_key_to_pem "$1"
}

# Generate a random key (32 bytes)
#
# random_data may contain the null byte (00) or newline (0a). `softhsm2-util --aes` uses fgets to read
# the key. It stops reading when encountering a null byte or newline. Therefore these are stripped out.
#
# $1 name of the key
function generate_wp_random_key {
    echo -e "${INFO}Generating random wallet provider key${NC}"
    # Replace '00' and '0a' by fixed values 'X' and 'Y'
    random_bytes 32 | LC_ALL=C tr '\000\n' "XY" > "${TARGET_DIR}/wallet_provider/$1.key"
}

# Generate a random key (64 bytes)
function generate_ws_random_key {
    echo -e "${INFO}Generating random wallet server key${NC}"
    random_bytes 64 > "${TARGET_DIR}/mock_relying_party/$1.key"
}

# Generate an EC root CA for the pid_issuer
function generate_pid_issuer_root_ca {
    echo -e "${INFO}Generating PID CA key pair${NC}"
    cargo run --manifest-path "${BASE_DIR}"/wallet_core/Cargo.toml --bin wallet_ca ca \
        --common-name "ca.pid.example.com" \
        --file-prefix "${TARGET_DIR}/pid_issuer/ca" \
        --force
}

# Generate an EC key pair for the pid_issuer
function generate_pid_issuer_key_pair {
    echo -e "${INFO}Generating PID Issuer key pair${NC}"
    cargo run --manifest-path "${BASE_DIR}"/wallet_core/Cargo.toml \
        --bin wallet_ca issuer \
        --ca-key-file "${TARGET_DIR}/pid_issuer/ca.key.pem" \
        --ca-crt-file "${TARGET_DIR}/pid_issuer/ca.crt.pem" \
        --common-name "pid.example.com" \
        --issuer-auth-file "${DEVENV}/rvig_issuer_auth.json" \
        --file-prefix "${TARGET_DIR}/pid_issuer/issuer" \
        --force

    openssl pkcs8 -topk8 -inform PEM -outform DER \
        -in "${TARGET_DIR}/pid_issuer/issuer.key.pem" -out "${TARGET_DIR}/pid_issuer/issuer_key.der" -nocrypt

    openssl x509 -in "${TARGET_DIR}/pid_issuer/issuer.crt.pem" \
        -outform der -out "${TARGET_DIR}/pid_issuer/issuer_crt.der"
}

# Generate an EC root CA for the mock_relying_party
function generate_mock_relying_party_root_ca {
    cargo run --manifest-path "${BASE_DIR}"/wallet_core/Cargo.toml --bin wallet_ca ca \
        --common-name "ca.rp.example.com" \
        --file-prefix "${TARGET_DIR}/mock_relying_party/ca" \
        --force
}

# Generate an EC key pair for the mock_relying_party
#
# $1 - READER_NAME: Name of the Relying Party
function generate_mock_relying_party_key_pair {
    cargo run --manifest-path "${BASE_DIR}"/wallet_core/Cargo.toml \
        --bin wallet_ca reader \
        --ca-key-file "${TARGET_DIR}/mock_relying_party/ca.key.pem" \
        --ca-crt-file "${TARGET_DIR}/mock_relying_party/ca.crt.pem" \
        --common-name "$1.example.com" \
        --reader-auth-file "${DEVENV}/$1_reader_auth.json" \
        --file-prefix "${TARGET_DIR}/mock_relying_party/$1" \
        --force

    openssl x509 -in "${TARGET_DIR}/mock_relying_party/$1.crt.pem" \
        -outform der -out "${TARGET_DIR}/mock_relying_party/$1.crt.der"

    openssl pkcs8 -topk8 -inform PEM -outform DER \
        -in "${TARGET_DIR}/mock_relying_party/$1.key.pem" -out "${TARGET_DIR}/mock_relying_party/$1.key.der" -nocrypt
}

function encrypt_gba_v_responses {
    mkdir -p "${GBA_HC_CONVERTER_DIR}/resources/encrypted-gba-v-responses"
    for file in "${GBA_HC_CONVERTER_DIR}"/resources/gba-v-responses/*; do
        if [ -f "$file" ]; then
            cargo run --manifest-path "${BASE_DIR}"/wallet_core/Cargo.toml \
                --bin gba_encrypt -- \
                --basename "$(basename "$file" .xml)" \
                --output "${GBA_HC_CONVERTER_DIR}/resources/encrypted-gba-v-responses" \
                "$file"
        fi
    done
}
