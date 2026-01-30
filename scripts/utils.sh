#!/usr/bin/env bash

########################################################################
# Globals, Includes
########################################################################

BASE64="openssl base64 -e -A"
SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"
BASE_DIR="$(dirname "${SCRIPTS_DIR}")"
source "${SCRIPTS_DIR}/colors.sh"

########################################################################
# Colors
########################################################################

SECTION=${LIGHT_BLUE}
SUCCESS=${LIGHT_GREEN}
ERROR=${RED}
WARN=${ORANGE}
INFO=${PURPLE}

########################################################################
# Functions
########################################################################

# Is this macOS?
function is_macos() {
  uname -a | grep -i darwin >/dev/null
}

# Print to stderr.
function error() {
  local msg=$1
  echo -e "${ERROR}$msg${NC}" 1>&2
}

# Print to stderr.
function warn() {
  local msg=$1
  echo -e "${WARN}$msg${NC}" 1>&2
}

# Prints the installable belonging to an `executable` ($1).
# If $1 is not in known_installables_map, it returns $1.
function map_installable() {

    local installable=$1

    # Mapping of executables to installable packages in the form:
    # "<EXECUTABLE>:<INSTALLABLE>".
    known_installables_map=(
        "cargo-set-version:cargo-edit"
        "cargo-hack:cargo-hack"
        "p11tool:gnutls"
    )

    for mapping in "${known_installables_map[@]}"; do
        local key="${mapping%%:*}"
        local value="${mapping##*:}"

        if [[ "$key" == "$installable" ]]; then
            installable=("$value")
        fi
    done

    echo "${installable[@]}"
}

# Check if required executables are available on the path.
function have() {
    local missing=()
    for executable in "$@"; do
        if [[ "$executable" == cargo-* ]]; then
            local subcommand="${executable#cargo-}"
            if ! cargo "$subcommand" --help &>/dev/null; then
                missing+=($(map_installable "$executable"))
            fi
        else
            which "$executable" &>/dev/null || missing+=($(map_installable "$executable"))
        fi
    done
    if [ ${#missing[@]} -eq 0 ]; then
        return 0
    else
        error "Missing required tool(s) to run this: ${missing[*]}"
        exit 1
    fi
}

function base64_url_encode() {
    ${BASE64} | tr '/+' '_-' | tr -d '=\n';
}

function detect_softhsm() {
  # shellcheck disable=SC2206
  local locations=("/usr/local/lib" ${NIX_PROFILES:-} ${nativeBuildInputs:-} "${HOMEBREW_PREFIX:+${HOMEBREW_PREFIX}/lib}" "/usr/lib" "/opt/softhsm")

  for location in "${locations[@]}"; do
      local library_path
      if [[ -d $location ]]; then
        library_path=$(find -L "$location" -maxdepth 3 -name "libsofthsm2.so" -or -name "libsofthsm2.dylib" | head -n 1 || true)
        if [[ -n $library_path ]]; then
            echo $library_path
            return
        fi
      fi
  done
  warn "Could not find SoftHSM shared library (libsofthsm2.so or libsofthsm2.dylib)"
}

function check_openssl() {
  if ! openssl version | grep -q "OpenSSL"
  then
    error "Please install an actual, real OpenSSL version"
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

# Generate or re-use by linking, a key and certificate to use as root CA.
#
# $1 - Target directory
# $2 - Common name
#
# If USE_SINGLE_CA is "1", and USE_SINGLE_CA_PATH is set to some file
# system path, the function will check the location of USE_SINGLE_CA_PATH
# and if a CA exists there, it will link to that instead of generate a
# new CA. This allows us to switch between single and multiple CA usage
# transparently. Single CA mode is convenient when doing API traffic
# inspection where the proxy used uses the same CA to generate in-between
# certificates to record traffic.
function generate_or_reuse_root_ca {

    # If single ca is wanted, and exists already, and target dir
    # is not equal to single ca path, link single ca to target:
    if [[ ${USE_SINGLE_CA} == 1 && -f ${USE_SINGLE_CA_PATH}/ca.crt.pem && "$1" != "${USE_SINGLE_CA_PATH}" ]]; then
        echo -e "${INFO}Single CA exists, re-using by linking files to $1 ${NC}"
        mkdir -p "$1"
        ln -sf "${USE_SINGLE_CA_PATH}"/ca.{key,crt}.{pem,der} "$1/"

    # If target ca files already exist:
    elif [[ -f ${1}/ca.crt.pem ]]; then
        echo -e "${INFO}CA files in $1 already exist, not (re-)generating${NC}"

    # Else just create:
    else
        echo -e "${INFO}Generating CA $2 in ${1}${NC}"
        mkdir -p "$1"

        # Note: 26 hours older than "now" is not arbitrary, but
        # required for usage with devproxy. For background, see:
        # https://github.com/dotnet/dev-proxy/issues/1410
        DATE_FORMAT='+%Y%m%d%H%M%SZ'
        if is_macos; then
            DATE_FROM="$(date -v -26H "$DATE_FORMAT")"
            DATE_TO="$(date -v +1y "$DATE_FORMAT")"
        else
            DATE_FROM="$(date --date='-26 hours' "$DATE_FORMAT")"
            DATE_TO="$(date --date='+1 year' "$DATE_FORMAT")"
        fi

        openssl req -x509 -sha256 -nodes -newkey rsa:2048 -subj "/CN=$2" -addext "keyUsage=critical,keyCertSign,cRLSign" -addext "basicConstraints=critical,CA:TRUE,pathlen:0" -not_before "$DATE_FROM" -not_after "$DATE_TO" -keyout "$1/ca.key.pem" -out "$1/ca.crt.pem"
        openssl pkcs8 -topk8 -inform PEM -outform DER -nocrypt -in "$1/ca.key.pem" -out "$1/ca.key.der"
        openssl x509 -in "$1/ca.crt.pem" -outform DER -out "$1/ca.crt.der"
        openssl pkcs12 -export -out "$1/ca.pfx" -inkey "$1/ca.key.pem" -in "$1/ca.crt.pem" -password pass:
    fi
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
            -outform DER -out "$1/$2.crt.der"

    echo -e "${INFO}Converting SSL private key to DER${NC}"
    openssl pkcs8 -topk8 -inform PEM -outform DER \
            -in "$1/$2.key" -out "$1/$2.key.der" -nocrypt
}

# Generate a private/public key pair in the HSM
#
# $1 name of the key
# $2 output path for public key relative to $TARGET_DIR
function generate_hsm_key_pair {
    # Generate EC key pair in the HSM
    p11tool \
        --provider "${HSM_LIBRARY_PATH}" \
        --login \
        --set-pin "${HSM_USER_PIN}" \
        --label "$1" \
        --generate-ecc \
        --curve secp256r1 \
        --outfile "${TARGET_DIR}/$2"
}

# Generate a private key in the HSM
#
# $1 name of the key
function generate_wp_signing_key {
    echo -e "${INFO}Generating HSM private key${NC}"

    generate_hsm_key_pair "$1_key" "wallet_provider/$1.pub.pem"

    openssl pkey -in "${TARGET_DIR}/wallet_provider/$1.pub.pem" -pubin \
        -outform DER -out "${TARGET_DIR}/wallet_provider/$1.pub.der"
}

# Generate a random AES key (32 bytes)
#
# random_data may contain the null byte (00) or newline (0a). `softhsm2-util --aes` uses fgets to read
# the key. It stops reading when encountering a null byte or newline. Therefore these are stripped out.
# This bug is fixed in newer versions: https://github.com/softhsm/SoftHSMv2/issues/746
#
# $1 name of the key
function generate_wp_aes_key {
    echo -e "${INFO}Generating AES wallet provider key${NC}"
    # Replace '00' and '0a' by fixed values 'X' and 'Y'
    random_bytes 32 | LC_ALL=C tr '\000\n' "XY" > "${TARGET_DIR}/wallet_provider/$1.key"
}

# Generate an EC root CA for issuer
function generate_issuer_root_ca {
    echo -e "${INFO}Generating Issuer CA key pair${NC}"
    cargo run --manifest-path "${BASE_DIR}"/wallet_core/Cargo.toml --bin wallet_ca ca \
        --common-name "ca.issuer.example.com" \
        --file-prefix "${TARGET_DIR}/ca.issuer" \
        --force
    openssl x509 -in "${TARGET_DIR}/ca.issuer.crt.pem" -outform DER -out "${TARGET_DIR}/ca.issuer.crt.der"
}

# Generate an EC root CA for reader
function generate_reader_root_ca {
    echo -e "${INFO}Generating Reader CA key pair${NC}"
    cargo run --manifest-path "${BASE_DIR}"/wallet_core/Cargo.toml --bin wallet_ca ca \
        --common-name "ca.reader.example.com" \
        --file-prefix "${TARGET_DIR}/ca.reader" \
        --force
    openssl x509 -in "${TARGET_DIR}/ca.reader.crt.pem" -outform DER -out "${TARGET_DIR}/ca.reader.crt.der"
}

# Generate an EC key pair for the config_signing
function generate_config_signing_key_pair {
    echo -e "${INFO}Generating config signing key pair${NC}"

    openssl ecparam -genkey -name prime256v1 -noout \
        -out "${TARGET_DIR}/wallet_provider/config_signing.ec.key" > /dev/null
    openssl pkcs8 -topk8 -inform PEM \
        -in "${TARGET_DIR}/wallet_provider/config_signing.ec.key" \
        -out "${TARGET_DIR}/wallet_provider/config_signing.pem" -nocrypt

    openssl ec -pubout \
        -in "${TARGET_DIR}/wallet_provider/config_signing.ec.key" \
        -out "${TARGET_DIR}/wallet_provider/config_signing.pub.pem"
    openssl pkey -pubin -outform DER \
        -in "${TARGET_DIR}/wallet_provider/config_signing.pub.pem" \
        -out "${TARGET_DIR}/wallet_provider/config_signing.pub.der"
}

# Generate an EC key pair for the pid_issuer
function generate_pid_issuer_key_pair {
    echo -e "${INFO}Generating PID Issuer key pair in HSM${NC}"

    generate_hsm_key_pair pid_issuer_key pid_issuer/issuer.pub.pem

    # Generate a certificate for the public key
    cargo run --manifest-path "${BASE_DIR}"/wallet_core/Cargo.toml \
        --bin wallet_ca issuer-cert \
        --public-key-file "${TARGET_DIR}/pid_issuer/issuer.pub.pem" \
        --ca-key-file "${TARGET_DIR}/ca.issuer.key.pem" \
        --ca-crt-file "${TARGET_DIR}/ca.issuer.crt.pem" \
        --common-name "pid.example.com" \
        --issuer-auth-file "${DEVENV}/rvig_issuer_auth.json" \
        --file-prefix "${TARGET_DIR}/pid_issuer/issuer" \
        --force

    # Convert the PEM certificate to DER format
    openssl x509 -in "${TARGET_DIR}/pid_issuer/issuer.crt.pem" \
        -outform DER -out "${TARGET_DIR}/pid_issuer/issuer.crt.der"
}

# Generate an EC key pair for the pid_issuer
function generate_pid_issuer_tsl_key_pair {
    echo -e "${INFO}Generating PID Issuer TSL key pair${NC}"

    # Generate a certificate for the public key including issuer authentication
    cargo run --manifest-path "${BASE_DIR}"/wallet_core/Cargo.toml \
        --bin wallet_ca tsl \
        --ca-key-file "${TARGET_DIR}/ca.issuer.key.pem" \
        --ca-crt-file "${TARGET_DIR}/ca.issuer.crt.pem" \
        --common-name "pid.example.com" \
        --file-prefix "${TARGET_DIR}/pid_issuer/tsl" \
        --force

    # Convert the PEM key to DER format
    openssl pkcs8 -topk8 -inform PEM -outform DER \
        -in "${TARGET_DIR}/pid_issuer/tsl.key.pem" \
        -out "${TARGET_DIR}/pid_issuer/tsl.key.der" -nocrypt

    # Convert the PEM certificate to DER format
    openssl x509 -in "${TARGET_DIR}/pid_issuer/tsl.crt.pem" \
        -outform DER -out "${TARGET_DIR}/pid_issuer/tsl.crt.der"
}

# Generate an EC key pair for the wallet_provider
function generate_wallet_provider_tsl_key_pair {
    echo -e "${INFO}Generating Wallet Provider WUA TSL key pair${NC}"

    generate_hsm_key_pair wua_tsl_key wallet_provider/wua_tsl.pub.pem

    # Generate a certificate for the public key including issuer authentication
    cargo run --manifest-path "${BASE_DIR}"/wallet_core/Cargo.toml \
        --bin wallet_ca tsl-cert \
        --public-key-file "${TARGET_DIR}/wallet_provider/wua_tsl.pub.pem" \
        --ca-key-file "${TARGET_DIR}/ca.issuer.key.pem" \
        --ca-crt-file "${TARGET_DIR}/ca.issuer.crt.pem" \
        --common-name "wua-issuer.example.com" \
        --file-prefix "${TARGET_DIR}/wallet_provider/wua_tsl" \
        --force

    # Convert the PEM certificate to DER format
    openssl x509 -in "${TARGET_DIR}/wallet_provider/wua_tsl.crt.pem" \
        -outform DER -out "${TARGET_DIR}/wallet_provider/wua_tsl.crt.der"
}

# Generate an EC key pairs for the demo_issuer
#
# $1 - ISSUER_NAME: Name of the Issuer
function generate_demo_issuer_key_pairs {
    cargo run --manifest-path "${BASE_DIR}"/wallet_core/Cargo.toml \
        --bin wallet_ca reader \
        --ca-key-file "${TARGET_DIR}/ca.reader.key.pem" \
        --ca-crt-file "${TARGET_DIR}/ca.reader.crt.pem" \
        --common-name "$1.example.com" \
        --reader-auth-file "${DEVENV}/$1_reader_auth.json" \
        --file-prefix "${TARGET_DIR}/demo_issuer/$1.reader" \
        --force

    cargo run --manifest-path "${BASE_DIR}"/wallet_core/Cargo.toml \
        --bin wallet_ca issuer \
        --ca-key-file "${TARGET_DIR}/ca.issuer.key.pem" \
        --ca-crt-file "${TARGET_DIR}/ca.issuer.crt.pem" \
        --common-name "$1.example.com" \
        --issuer-auth-file "${DEVENV}/$1_issuer_auth.json" \
        --file-prefix "${TARGET_DIR}/demo_issuer/$1.issuer" \
        --force

    cargo run --manifest-path "${BASE_DIR}"/wallet_core/Cargo.toml \
        --bin wallet_ca tsl \
        --ca-key-file "${TARGET_DIR}/ca.issuer.key.pem" \
        --ca-crt-file "${TARGET_DIR}/ca.issuer.crt.pem" \
        --common-name "$1.example.com" \
        --file-prefix "${TARGET_DIR}/demo_issuer/$1.tsl" \
        --force

    openssl x509 -in "${TARGET_DIR}/demo_issuer/$1.reader.crt.pem" \
        -outform DER -out "${TARGET_DIR}/demo_issuer/$1.reader.crt.der"
    openssl x509 -in "${TARGET_DIR}/demo_issuer/$1.issuer.crt.pem" \
        -outform DER -out "${TARGET_DIR}/demo_issuer/$1.issuer.crt.der"
    openssl x509 -in "${TARGET_DIR}/demo_issuer/$1.tsl.crt.pem" \
        -outform DER -out "${TARGET_DIR}/demo_issuer/$1.tsl.crt.der"

    openssl pkcs8 -topk8 -inform PEM -outform DER \
        -in "${TARGET_DIR}/demo_issuer/$1.reader.key.pem" \
        -out "${TARGET_DIR}/demo_issuer/$1.reader.key.der" -nocrypt
    openssl pkcs8 -topk8 -inform PEM -outform DER \
        -in "${TARGET_DIR}/demo_issuer/$1.issuer.key.pem" \
        -out "${TARGET_DIR}/demo_issuer/$1.issuer.key.der" -nocrypt
    openssl pkcs8 -topk8 -inform PEM -outform DER \
        -in "${TARGET_DIR}/demo_issuer/$1.tsl.key.pem" \
        -out "${TARGET_DIR}/demo_issuer/$1.tsl.key.der" -nocrypt
}

# Generate an EC key pair for the demo_relying_party
#
# $1 - RELYING_PARTY_NAME: Name of the Relying Party
function generate_demo_relying_party_key_pair {
    cargo run --manifest-path "${BASE_DIR}"/wallet_core/Cargo.toml \
        --bin wallet_ca reader \
        --ca-key-file "${TARGET_DIR}/ca.reader.key.pem" \
        --ca-crt-file "${TARGET_DIR}/ca.reader.crt.pem" \
        --common-name "$1.example.com" \
        --reader-auth-file "${DEVENV}/$1_reader_auth.json" \
        --file-prefix "${TARGET_DIR}/demo_relying_party/$1" \
        --force

    openssl x509 -in "${TARGET_DIR}/demo_relying_party/$1.crt.pem" \
        -outform DER -out "${TARGET_DIR}/demo_relying_party/$1.crt.der"

    openssl pkcs8 -topk8 -inform PEM -outform DER \
        -in "${TARGET_DIR}/demo_relying_party/$1.key.pem" -out "${TARGET_DIR}/demo_relying_party/$1.key.der" -nocrypt
}

# Generate an EC key pair for the demo_relying_party in the HSM.
# The label of the key is "${READER_NAME}_key"
#
# $1 - READER_NAME: Name of the Relying Party
# $2 - path where the certificate will be written to
function generate_relying_party_hsm_key_pair {
    # Generate EC key pair in the HSM
    generate_hsm_key_pair "$1_key" "$2/$1.pub.pem"

    # Generate a certificate for the public key including reader authentication
    cargo run --manifest-path "${BASE_DIR}"/wallet_core/Cargo.toml \
          --bin wallet_ca reader-cert \
          --public-key-file "${TARGET_DIR}/$2/$1.pub.pem" \
          --ca-key-file "${TARGET_DIR}/ca.reader.key.pem" \
          --ca-crt-file "${TARGET_DIR}/ca.reader.crt.pem" \
          --common-name "$1.example.com" \
          --reader-auth-file "${DEVENV}/$1_reader_auth.json" \
          --file-prefix "${TARGET_DIR}/$2/$1" \
          --force

    # Convert the PEM certificate to DER format
    openssl x509 -in "${TARGET_DIR}/$2/$1.crt.pem" \
        -outform DER -out "${TARGET_DIR}/$2/$1.crt.der"
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
