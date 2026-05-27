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
IOS_ENV_NAME="${IOS_ENV_NAME:-}"
IOS_ENV_SLUG="${IOS_ENV_SLUG:-}"
APPLICATION_ID="${APPLICATION_ID:-}"
APP_NAME="${APP_NAME:-}"
WALLET_CONFIG_DIR="${WALLET_CONFIG_DIR:-${BASE_DIR}/wallet_core/wallet}"
UL_HOSTNAME="${UL_HOSTNAME:-}"
APPLE_ATTESTATION_ENVIRONMENT="${APPLE_ATTESTATION_ENVIRONMENT:-}"
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
    "./scripts/run-ios-ont.sh or ./scripts/run-ios-demo.sh"

require_env_vars \
    PIPELINE_SOURCE \
    PIPELINE_DESCRIPTION \
    IOS_ENV_NAME \
    IOS_ENV_SLUG \
    APPLICATION_ID \
    APP_NAME \
    UL_HOSTNAME \
    APPLE_ATTESTATION_ENVIRONMENT

CONFIG_ENV="${IOS_ENV_SLUG}"
WALLET_CONFIG_JOB_NAME="wallet-config-${IOS_ENV_SLUG}"
export FLUTTER_XCODE_BUNDLE_IDENTIFIER="${APPLICATION_ID}"
export FLUTTER_XCODE_APP_NAME="${APP_NAME}"

have flutter glab jq xcrun

function check_ios_device_visibility() {
    local devices_json
    local device_id="${1:-}"

    devices_json="$(xcrun xcdevice list 2>/dev/null || true)"
    if [[ -z "${devices_json}" ]]; then
        error "Xcode did not return any device information."
        cat 1>&2 <<EOF
Check the device connection in Xcode first:
  1. Open Xcode > Window > Devices and Simulators
  2. Unlock the iPhone and reconnect the cable if needed
  3. Accept any trust / developer mode prompts on the device
EOF
        exit 1
    fi

    if ! echo "${devices_json}" | jq -e 'type == "array"' >/dev/null 2>&1; then
        error "Unexpected output from 'xcrun xcdevice list'."
        echo "${devices_json}" 1>&2
        exit 1
    fi

    if [[ -n "${device_id}" ]]; then
        if ! echo "${devices_json}" | jq -e --arg id "${device_id}" '
            any(.[]; .simulator == false and (.platform | startswith("com.apple.platform.iphone")) and .available == true and .identifier == $id)
        ' >/dev/null; then
            error "Requested iOS device '${device_id}' is not visible to Xcode."
            cat 1>&2 <<EOF
Verify that Xcode can see the phone in Window > Devices and Simulators.
If it is connected but unavailable, unlock it and accept trust / developer mode prompts.
EOF
            exit 1
        fi

        return 0
    fi

    if ! echo "${devices_json}" | jq -e '
        any(.[]; .simulator == false and (.platform | startswith("com.apple.platform.iphone")) and .available == true)
    ' >/dev/null; then
        error "No physical iPhone is currently visible to Xcode."
        cat 1>&2 <<EOF
This script targets ${IOS_ENV_NAME} on a real device. Open Xcode > Window > Devices and Simulators,
then make sure the phone is unlocked, trusted and has Developer Mode enabled.
EOF
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
export APPLE_ATTESTATION_ENVIRONMENT

require_env_vars UNIVERSAL_LINK_BASE

echo
echo -e "${SECTION}Resolved ${IOS_ENV_NAME} iOS configuration${NC}"
echo -e "  CONFIG_ENV=${CYAN}${CONFIG_ENV}${NC}"
echo -e "  WALLET_CONFIG_DIR=${CYAN}${WALLET_CONFIG_DIR}${NC}"
echo -e "  UL_HOSTNAME=${CYAN}${UL_HOSTNAME}${NC}"
echo -e "  UNIVERSAL_LINK_BASE=${CYAN}${UNIVERSAL_LINK_BASE}${NC}"
echo -e "  APPLE_ATTESTATION_ENVIRONMENT=${CYAN}${APPLE_ATTESTATION_ENVIRONMENT}${NC}"
echo -e "  APPLICATION_ID=${CYAN}${APPLICATION_ID}${NC}"
echo -e "  APP_NAME=${CYAN}${APP_NAME}${NC}"
echo -e "${WARN}Use a physical iOS device. The simulator uses faux attestation and ${IOS_ENV_NAME} will reject it.${NC}"

echo
echo -e "${SECTION}Running Flutter on iOS${NC}"
check_ios_device_visibility "$(requested_device_id || true)"

cd "${BASE_DIR}/wallet_app"
exec flutter run \
    --release \
    --dart-define=MOCK_REPOSITORIES="${MOCK_REPOSITORIES}" \
    --dart-define=SHOW_DEBUG_OPTIONS="${SHOW_DEBUG_OPTIONS}" \
    --dart-define=UL_HOSTNAME="${UL_HOSTNAME}" \
    --dart-define=DEMO_INDEX_URL="${DEMO_INDEX_URL}" \
    "${EXTRA_ARGS[@]}"
