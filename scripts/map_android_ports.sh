#!/usr/bin/env bash

SCRIPTS_DIR=$(dirname "$(realpath "$(command -v "${BASH_SOURCE[0]}")")")
export SCRIPTS_DIR

source "${SCRIPTS_DIR}/configuration.sh"

set -eu

if command -v adb > /dev/null
then
    echo -e "Mapping Android ports with ${GREEN}adb${NC}"
    if adb reverse tcp:${WALLET_PROVIDER_PORT} tcp:${WALLET_PROVIDER_PORT}
    then
        adb reverse tcp:${CONFIG_SERVER_PORT} tcp:${CONFIG_SERVER_PORT}
        adb reverse tcp:${UPDATE_POLICY_SERVER_PORT} tcp:${UPDATE_POLICY_SERVER_PORT}
        adb reverse tcp:${PID_ISSUER_WS_PORT} tcp:${PID_ISSUER_WS_PORT}
        adb reverse tcp:${DEMO_INDEX_PORT} tcp:${DEMO_INDEX_PORT}
        adb reverse tcp:${DEMO_ISSUER_PORT} tcp:${DEMO_ISSUER_PORT}
        adb reverse tcp:${ISSUANCE_SERVER_WS_PORT} tcp:${ISSUANCE_SERVER_WS_PORT}
        adb reverse tcp:${DEMO_RP_PORT} tcp:${DEMO_RP_PORT}
        adb reverse tcp:${VERIFICATION_SERVER_WS_PORT} tcp:${VERIFICATION_SERVER_WS_PORT}
        adb reverse tcp:${RDO_MAX_PORT} tcp:${RDO_MAX_PORT}
    else
        echo -e "Please start the Android emulator, and run ${BLUE}$0 $@${NC} again"
    fi
else
    echo -e "Android ${GREEN}adb${NC} command not found"
fi
