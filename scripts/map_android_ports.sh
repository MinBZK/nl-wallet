#!/usr/bin/env bash

export SCRIPTS_DIR=$(dirname $(realpath $(command -v ${BASH_SOURCE[0]})))

source "${SCRIPTS_DIR}/configuration.sh"

if command -v adb > /dev/null
then
    echo -e "Mapping Android ports with ${GREEN}adb${NC}"
    if adb reverse tcp:${WALLET_PROVIDER_PORT} tcp:${WALLET_PROVIDER_PORT}
    then
        adb reverse tcp:${MOCK_RP_WS_PORT} tcp:${MOCK_RP_WS_PORT}
        adb reverse tcp:${MOCK_RP_RS_PORT} tcp:${MOCK_RP_RS_PORT}
        adb reverse tcp:${MOCK_RP_PORT} tcp:${MOCK_RP_PORT}
        adb reverse tcp:${CONFIG_SERVER_PORT} tcp:${CONFIG_SERVER_PORT}
        adb reverse tcp:${PID_ISSUER_WS_PORT} tcp:${PID_ISSUER_WS_PORT}
        adb reverse tcp:${RDO_MAX_PORT} tcp:${RDO_MAX_PORT}
    else
        echo -e "Please start the Android emulator, and run ${BLUE}$0 $@${NC} again"
    fi
else
    echo -e "Android ${GREEN}adb${NC} command not found"
fi
