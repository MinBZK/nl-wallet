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
  local locations=("/usr/local/lib" ${NIX_PROFILES:-} "${HOMEBREW_PREFIX:+${HOMEBREW_PREFIX}/lib}" "/usr/lib/")

  for location in "${locations[@]}"; do
      local library_path
      library_path=$(find -L "$location" -name "libsofthsm2.so" | head -n 1)
      if [ -n "$library_path" ]; then
          echo "$library_path"
          return
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

# Generate n random bytes.
#
# $1 n: how many random bytes to generate
function random_bytes {
    dd if=/dev/urandom bs="$1" count=1 2>/dev/null
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

# Generate a random key (32 bytes)
#
# random_data may contain the null byte (00) or newline (0a). `softhsm2-util --aes` uses fgets to read
# the key. It stops reading when encountering a null byte or newline. Therefore these are stripped out.
#
# $1 name of the key
function generate_wp_random_key {
    echo -e "${INFO}Generating random key${NC}"
    # Replace '00' and '0a' by fixed values 'X' and 'Y'
    random_bytes 32 | LC_ALL=C tr '\000\n' "XY" > "${TARGET_DIR}/wallet_provider/$1.key"
}

# Generate an EC root CA for the pid_issuer
function generate_pid_issuer_root_ca {
    echo -e "${INFO}Generating PID root CA${NC}"
    openssl req -x509 \
            -nodes \
            -newkey ec \
            -pkeyopt ec_paramgen_curve:prime256v1 \
            -keyout "${TARGET_DIR}/pid_issuer/ca_privkey.pem" \
            -out "${TARGET_DIR}/pid_issuer/ca_cert.pem" \
            -days 365 \
            -addext keyUsage=keyCertSign,cRLSign \
            -subj '/CN=ca.example.com'

    openssl x509 -in "${TARGET_DIR}/pid_issuer/ca_cert.pem" \
            -outform der -out "${TARGET_DIR}/pid_issuer/ca_cert.der"
}

# Generate an EC key pair for the pid_issuer
function generate_pid_issuer_key_pair {
    echo -e "${INFO}Generating PID issuer private key and CSR${NC}"
    openssl req -new \
            -nodes \
            -newkey ec \
            -pkeyopt ec_paramgen_curve:prime256v1 \
            -keyout "${TARGET_DIR}/pid_issuer/issuer_key.pem" \
            -out "${TARGET_DIR}/pid_issuer/issuer_csr.pem" \
            -subj "/CN=pid.example.com"

    echo -e "${INFO}Generate PID certificate from CSR using root CA${NC}"
    openssl x509 -req \
            -extfile <(printf "keyUsage=digitalSignature\nextendedKeyUsage=1.0.18013.5.1.2\nbasicConstraints=CA:FALSE") \
            -in "${TARGET_DIR}/pid_issuer/issuer_csr.pem" \
            -days 500 \
            -CA "${TARGET_DIR}/pid_issuer/ca_cert.pem" \
            -CAkey "${TARGET_DIR}/pid_issuer/ca_privkey.pem" \
            -out "${TARGET_DIR}/pid_issuer/issuer_crt.pem"

    openssl pkcs8 -topk8 -inform PEM -outform DER \
        -in "${TARGET_DIR}/pid_issuer/issuer_key.pem" -out "${TARGET_DIR}/pid_issuer/issuer_key.der" -nocrypt

    openssl x509 -in "${TARGET_DIR}/pid_issuer/issuer_crt.pem" \
        -outform der -out "${TARGET_DIR}/pid_issuer/issuer_crt.der"
}

# Generate an EC root CA for the mock_relying_party
function generate_mock_relying_party_root_ca {
    cargo run --manifest-path "${BASE_DIR}"/wallet_core/Cargo.toml --bin wallet_ca ca \
        --common-name "ca.example.com" \
        --file-prefix "${TARGET_DIR}/mock_relying_party/ca" \
        --force

    openssl x509 -in "${TARGET_DIR}/mock_relying_party/ca.crt.pem" \
        -outform der -out "${TARGET_DIR}/mock_relying_party/ca.crt.der"
}

# Generate an EC key pair for the mock_relying_party
function generate_mock_relying_party_key_pair {
    cargo run --manifest-path "${BASE_DIR}"/wallet_core/Cargo.toml --bin wallet_ca reader-auth-cert \
        --ca-key-file "${TARGET_DIR}/mock_relying_party/ca.key.pem" \
        --ca-crt-file "${TARGET_DIR}/mock_relying_party/ca.crt.pem" \
        --common-name "rp.example.com" \
        --reader-auth-file "${DEVENV}/reader_auth.json" \
        --file-prefix "${TARGET_DIR}/mock_relying_party/rp" \
        --force

    openssl x509 -in "${TARGET_DIR}/mock_relying_party/rp.crt.pem" \
        -outform der -out "${TARGET_DIR}/mock_relying_party/rp.crt.der"

    openssl pkcs8 -topk8 -inform PEM -outform DER \
        -in "${TARGET_DIR}/mock_relying_party/rp.key.pem" -out "${TARGET_DIR}/mock_relying_party/rp.key.der" -nocrypt
}
