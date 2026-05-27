#!/usr/bin/env bash

set -euo pipefail

SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"
BASE_DIR="$(dirname "${SCRIPTS_DIR}")"
ENTRYPOINT_SCRIPT="${ENTRYPOINT_SCRIPT:-${BASH_SOURCE[0]}}"

source "${SCRIPTS_DIR}/run-functions.sh"

PROJECT_ID="${GITLAB_PROJECT_ID:-3345}"
GITLAB_REF="${GITLAB_REF:-main}"
PIPELINE_SOURCE="${PIPELINE_SOURCE:-}"
PIPELINE_DESCRIPTION="${PIPELINE_DESCRIPTION:-}"
ANDROID_ENV_NAME="${ANDROID_ENV_NAME:-}"
ANDROID_ENV_SLUG="${ANDROID_ENV_SLUG:-}"
APPLICATION_ID="${APPLICATION_ID:-}"
APP_NAME="${APP_NAME:-}"
WALLET_CONFIG_DIR="${WALLET_CONFIG_DIR:-${BASE_DIR}/wallet_core/wallet}"
UL_HOSTNAME="${UL_HOSTNAME:-}"
UNIVERSAL_LINK_PATH="${UNIVERSAL_LINK_PATH:-/deeplink/}"
SHOW_DEBUG_OPTIONS="${SHOW_DEBUG_OPTIONS:-true}"
MOCK_REPOSITORIES="${MOCK_REPOSITORIES:-false}"
DEMO_INDEX_URL="${DEMO_INDEX_URL:-}"
PIPELINE_ID="${PIPELINE_ID:-}"

EXTRA_ARGS=()

if [[ $# -gt 0 ]]; then
    if [[ "$1" != "--" ]]; then
        error "Unknown option '$1'. Use -- to pass arguments to flutter run."
        exit 1
    fi

    shift
    EXTRA_ARGS+=("$@")
fi

require_wrapper_entrypoint \
    "${BASH_SOURCE[0]}" \
    "${ENTRYPOINT_SCRIPT}" \
    "./scripts/run-android-ont.sh or ./scripts/run-android-demo.sh"

require_env_vars \
    PIPELINE_SOURCE \
    PIPELINE_DESCRIPTION \
    ANDROID_ENV_NAME \
    ANDROID_ENV_SLUG \
    APPLICATION_ID \
    APP_NAME \
    UL_HOSTNAME

CONFIG_ENV="${ANDROID_ENV_SLUG}"
WALLET_CONFIG_JOB_NAME="wallet-config-${ANDROID_ENV_SLUG}"

have flutter glab jq adb

function android_device_ids() {
    adb devices 2>/dev/null | awk 'NR > 1 && $2 == "device" && $1 !~ /^emulator-/ { print $1 }'
}

function check_android_device_visibility() {
    local device_id="${1:-}"
    local visible_device_ids=()

    adb start-server >/dev/null 2>&1 || true
    mapfile -t visible_device_ids < <(android_device_ids)

    if [[ -n "${device_id}" ]]; then
        if [[ ! " ${visible_device_ids[*]} " =~ [[:space:]]${device_id}[[:space:]] ]]; then
            error "Requested Android device '${device_id}' is not visible via adb."
            cat 1>&2 <<EOF
Verify the device connection first:
  1. Unlock the phone and reconnect the cable if needed
  2. Accept any USB debugging prompt on the device
  3. Check 'adb devices' and retry once it shows as 'device'
EOF
            exit 1
        fi

        return 0
    fi

    if [[ ${#visible_device_ids[@]} -eq 0 ]]; then
        error "No physical Android device is currently visible via adb."
        cat 1>&2 <<EOF
This script targets ${ANDROID_ENV_NAME} on a real device.
Unlock the phone, accept the USB debugging prompt and check 'adb devices'.
EOF
        exit 1
    fi
}

function read_android_signing_property() {
    local key="$1"
    local key_properties_path="${BASE_DIR}/wallet_app/android/key.properties"

    awk -F '=' -v key="${key}" '$1 == key { sub(/^[[:space:]]+/, "", $2); sub(/[[:space:]]+$/, "", $2); print $2; exit }' \
        "${key_properties_path}"
}

function check_android_signing_configuration() {
    local key_properties_path="${BASE_DIR}/wallet_app/android/key.properties"
    local store_file
    local store_file_path

    if [[ ! -f "${key_properties_path}" ]]; then
        error "Android signing config is missing at '${key_properties_path}'."
        cat 1>&2 <<EOF
Release Android runs need the local signing files in place:
  1. Put key.properties in wallet_app/android
  2. Put the referenced keystore file in wallet_app/android
EOF
        exit 1
    fi

    store_file="$(read_android_signing_property "storeFile")"
    if [[ -z "${store_file}" ]]; then
        error "Could not read 'storeFile' from '${key_properties_path}'."
        exit 1
    fi

    if [[ "${store_file}" = /* ]]; then
        store_file_path="${store_file}"
    else
        store_file_path="${BASE_DIR}/wallet_app/android/${store_file}"
    fi

    if [[ ! -f "${store_file_path}" ]]; then
        error "Android keystore '${store_file_path}' does not exist."
        exit 1
    fi
}

if [[ -z "${PIPELINE_ID}" ]]; then
    echo -e "${INFO}Resolving the latest successful ${PIPELINE_DESCRIPTION}${NC}"
    PIPELINE_ID="$(resolve_pipeline_id_for_jobs \
        "${PROJECT_ID}" \
        "${GITLAB_REF}" \
        "${PIPELINE_SOURCE}" \
        "${PIPELINE_DESCRIPTION}" \
        "${WALLET_CONFIG_JOB_NAME}")"
fi

echo -e "${INFO}Using pipeline ${CYAN}${PIPELINE_ID}${NC}${NC}"

JOBS_JSON="$(resolve_jobs_json "${PROJECT_ID}" "${PIPELINE_ID}")"

WALLET_CONFIG_JOB_ID="$(resolve_job_id "${JOBS_JSON}" "${WALLET_CONFIG_JOB_NAME}")"
echo -e "${INFO}Fetching ${CYAN}${WALLET_CONFIG_JOB_NAME}${NC} from job ${CYAN}${WALLET_CONFIG_JOB_ID}${NC}"
fetch_wallet_config "${PROJECT_ID}" "${WALLET_CONFIG_JOB_ID}" "${WALLET_CONFIG_DIR}"

UNIVERSAL_LINK_BASE="${UNIVERSAL_LINK_BASE:-https://${UL_HOSTNAME}${UNIVERSAL_LINK_PATH}}"
export CONFIG_ENV
export UL_HOSTNAME
export UNIVERSAL_LINK_BASE
export APPLICATION_ID
export APP_NAME

require_env_vars UNIVERSAL_LINK_BASE

echo
echo -e "${SECTION}Resolved ${ANDROID_ENV_NAME} Android configuration${NC}"
echo -e "  CONFIG_ENV=${CYAN}${CONFIG_ENV}${NC}"
echo -e "  WALLET_CONFIG_DIR=${CYAN}${WALLET_CONFIG_DIR}${NC}"
echo -e "  UL_HOSTNAME=${CYAN}${UL_HOSTNAME}${NC}"
echo -e "  UNIVERSAL_LINK_BASE=${CYAN}${UNIVERSAL_LINK_BASE}${NC}"
echo -e "  APPLICATION_ID=${CYAN}${APPLICATION_ID}${NC}"
echo -e "  APP_NAME=${CYAN}${APP_NAME}${NC}"
echo -e "${WARN}Use a physical Android device. Emulators will not pass attestation against ${ANDROID_ENV_NAME}.${NC}"

check_android_signing_configuration

echo
echo -e "${SECTION}Running Flutter on Android${NC}"
check_android_device_visibility "$(requested_device_id || true)"

cd "${BASE_DIR}/wallet_app"
exec flutter run \
    --release \
    --dart-define=MOCK_REPOSITORIES="${MOCK_REPOSITORIES}" \
    --dart-define=SHOW_DEBUG_OPTIONS="${SHOW_DEBUG_OPTIONS}" \
    --dart-define=UL_HOSTNAME="${UL_HOSTNAME}" \
    --dart-define=DEMO_INDEX_URL="${DEMO_INDEX_URL}" \
    "${EXTRA_ARGS[@]}"
