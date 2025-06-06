#!/usr/bin/env bash
# This script allows one to manage the individual services that are needed
# to run a completely local NL Wallet development environment.
#
# - nl-rdo-max (digid-connector)
# - demo_index
# - demo_relying_party
# - demo_issuer
# - verification_server
# - issuance_server
# - pid_issuer
# - wallet_provider
# - wallet

set -e # break on error
set -u # warn against undefined variables
set -o pipefail
# set -x # echo statements before executing, useful while debugging

########################################################################
# Globals and includes
########################################################################

SCRIPTS_DIR="$(cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd)"
BASE_DIR=$(dirname "${SCRIPTS_DIR}")
DOCKER_COMPOSE_FILE=${SCRIPTS_DIR}/docker-compose.yml

source "${SCRIPTS_DIR}/utils.sh"
source "${SCRIPTS_DIR}/configuration.sh"

########################################################################
# Functions
########################################################################

# Echo help information about this script
function usage() {
    echo -e "$(basename "${BASH_SOURCE[0]}"): Manage the Wallet Development environment

Usage: $(basename "${BASH_SOURCE[0]}") [OPTIONS] <SERVICES>

  Starts or restarts the services that are part of the development environment.

Where:

  SERVICE is any of:
    wallet:                     Start the wallet Flutter application.
                                This requires a simulator to be running.
    wp, wallet_provider:        Start the wallet_provider.
                                This requires a PostgreSQL database to be running, which can be provided by the
                                'docker' service.
    vs, verification_server:    Start the verification_server.
    is, issuance_server:        Start the issuance_server.
    pi, pid_issuer:             Start the pid_issuer.
    drp, demo_relying_party:    Start the demo_relying_party.
    di, demo_issuer:            Start the demo_issuer.
    dx, demo_index:             Start the demo_index.
    digid, digid_connector:     Start the digid_connector and a redis on docker.
    cs, configuration_server:   Start the configuration server
    ups, update_policy_server:  Start the update policy server
    brp:                        Start the Haal-Centraal BRP proxy with GBA HC converter.
    brpproxy:                   Start the Haal-Centraal BRP proxy.
    gba, gba_hc_converter:      Start the GBA HC converter.
    postgres:                   Start a PostgreSQL database, using Docker.

  OPTION is any of:
    --all                       Start all of the above services.
    --default                   Start all of the above services, excluding postgres and wallet.
                                This option is provided when a PostgreSQL database is run and managed by the user.
    --stop                      Combine with --all, --default or specific service name(s) to stop said services.
    -h, --help                  Show this help
"
}

########################################################################
# Check prerequisites
########################################################################

have cargo docker flutter

########################################################################
# Commandline arguments
########################################################################

DEMO_RELYING_PARTY=1
DEMO_ISSUER=1
DEMO_INDEX=1
WALLET_PROVIDER=1
VERIFICATION_SERVER=1
ISSUANCE_SERVER=1
PID_ISSUER=1
WALLET=1
DIGID_CONNECTOR=1
CONFIG_SERVER=1
UPDATE_POLICY_SERVER=1
BRP_PROXY=1
GBA_HC=1
POSTGRES=1

USAGE=1

STOP=0
START=0

if [ "$#" == "0" ]
then
    USAGE=0
fi

if [ "$#" == "1" ] && [ "$1" == "--stop" ]; then
    error "The --stop argument requires at least one service, --default, or --all to be specified."
    usage
    exit 1
fi

while [[ $# -gt 0 ]]
do
    case $1 in
        wallet)
            WALLET=0
            shift # past argument
            ;;
        wp|wallet_provider)
            WALLET_PROVIDER=0
            shift # past argument
            ;;
        vs|verification_server)
            VERIFICATION_SERVER=0
            shift # past argument
            ;;
        is|issuance_server)
            ISSUANCE_SERVER=0
            shift # past argument
            ;;
        pi|pid_issuer)
            PID_ISSUER=0
            shift # past argument
            ;;
        drp|demo_relying_party)
            DEMO_RELYING_PARTY=0
            shift # past argument
            ;;
        di|demo_issuer)
            DEMO_ISSUER=0
            shift # past argument
            ;;
        dx|demo_index)
            DEMO_INDEX=0
            shift # past argument
            ;;
        digid|digid_connector)
            DIGID_CONNECTOR=0
            shift # past argument
            ;;
        cs|configuration_server)
            CONFIG_SERVER=0
            shift
            ;;
        ups|update_policy_server)
            UPDATE_POLICY_SERVER=0
            shift
            ;;
        brp)
            BRP_PROXY=0
            GBA_HC=0
            shift
            ;;
        brpproxy)
            BRP_PROXY=0
            shift
            ;;
        gba|gba_hc_converter)
            GBA_HC=0
            shift
            ;;
        postgres)
            POSTGRES=0
            shift # past argument
            ;;
        --default)
            DIGID_CONNECTOR=0
            DEMO_RELYING_PARTY=0
            DEMO_INDEX=0
            DEMO_ISSUER=0
            VERIFICATION_SERVER=0
            ISSUANCE_SERVER=0
            PID_ISSUER=0
            WALLET_PROVIDER=0
            CONFIG_SERVER=0
            UPDATE_POLICY_SERVER=0
            BRP_PROXY=0
            GBA_HC=0
            shift # past argument
            ;;
        --all)
            DIGID_CONNECTOR=0
            POSTGRES=0
            DEMO_RELYING_PARTY=0
            DEMO_INDEX=0
            DEMO_ISSUER=0
            VERIFICATION_SERVER=0
            ISSUANCE_SERVER=0
            PID_ISSUER=0
            WALLET_PROVIDER=0
            WALLET=0
            CONFIG_SERVER=0
            UPDATE_POLICY_SERVER=0
            BRP_PROXY=0
            GBA_HC=0
            shift # past argument
            ;;
        -h|--help)
            USAGE=0
            shift # past argument
            ;;
        --stop)
            START=1
            shift # past argument
            ;;
        *)
            error "Unknown argument: $1"
            shift # past argument
            usage
            exit 1
            ;;
    esac
done

if [ "${USAGE}" == "0" ]
then
    usage
    exit 0
fi

########################################################################
# Manage digid-connector
########################################################################

if [ "${DIGID_CONNECTOR}" == "0" ]
then
    echo
    echo -e "${SECTION}Manage digid-connector${NC}"

    cd "${DIGID_CONNECTOR_PATH}"

    if [ "${STOP}" == "0" ]
    then
        echo -e "${INFO}Stopping ${ORANGE}digid-connector${NC}"
        docker compose down || true
    fi
    if [ "${START}" == "0" ]
    then
        echo -e "Building and starting ${ORANGE}digid-connector${NC}"
        docker compose up --detach --build --force-recreate
    fi
fi

########################################################################
# Manage postgres
########################################################################

if [ "${POSTGRES}" == "0" ]
then
    echo
    echo -e "${SECTION}Manage postgres services${NC}"

    if [ "${STOP}" == "0" ]
    then
        echo -e "${INFO}Stopping postgres services${NC}"
        docker compose --file "${DOCKER_COMPOSE_FILE}" down postgres || true
    fi
    if [ "${START}" == "0" ]
    then
        echo -e "${INFO}Starting postgres services${NC}"
        docker compose --file "${DOCKER_COMPOSE_FILE}" up --detach postgres
    fi
fi

########################################################################
# Manage demo_index
########################################################################

if [ "${DEMO_INDEX}" == "0" ]
then
    echo
    echo -e "${SECTION}Manage demo_index${NC}"

    cd "${DEMO_INDEX_DIR}"

    if [ "${STOP}" == "0" ]
    then
        echo -e "${INFO}Kill any running ${ORANGE}demo_index${NC}"
        killall demo_index || true
    fi
    if [ "${START}" == "0" ]
    then
        echo -e "${INFO}Start ${ORANGE}demo_index${NC}"
        RUST_LOG=debug cargo run --package demo_index --bin demo_index > "${TARGET_DIR}/demo_index.log" 2>&1 &

        echo -e "demo_index logs can be found at ${CYAN}${TARGET_DIR}/demo_index.log${NC}"
    fi
fi

########################################################################
# Manage demo_relying_party
########################################################################

if [ "${DEMO_RELYING_PARTY}" == "0" ]
then
    echo
    echo -e "${SECTION}Manage demo_relying_party${NC}"

    cd "${DEMO_RELYING_PARTY_DIR}"

    if [ "${STOP}" == "0" ]
    then
        echo -e "${INFO}Kill any running ${ORANGE}demo_relying_party${NC}"
        killall demo_relying_party || true
    fi
    if [ "${START}" == "0" ]
    then
        echo -e "${INFO}Start ${ORANGE}demo_relying_party${NC}"
        RUST_LOG=debug cargo run --package demo_relying_party --features "allow_insecure_url" --bin demo_relying_party > "${TARGET_DIR}/demo_relying_party.log" 2>&1 &

        echo -e "demo_relying_party logs can be found at ${CYAN}${TARGET_DIR}/demo_relying_party.log${NC}"
    fi
fi

########################################################################
# Manage demo_issuer
########################################################################

if [ "${DEMO_ISSUER}" == "0" ]
then
    echo
    echo -e "${SECTION}Manage demo_issuer${NC}"

    cd "${DEMO_ISSUER_DIR}"

    if [ "${STOP}" == "0" ]
    then
        echo -e "${INFO}Kill any running ${ORANGE}demo_issuer${NC}"
        killall demo_issuer || true
    fi
    if [ "${START}" == "0" ]
    then
        echo -e "${INFO}Start ${ORANGE}demo_issuer${NC}"
        RUST_LOG=debug cargo run --package demo_issuer --bin demo_issuer > "${TARGET_DIR}/demo_issuer.log" 2>&1 &

        echo -e "demo_issuer logs can be found at ${CYAN}${TARGET_DIR}/demo_issuer.log${NC}"
    fi
fi

########################################################################
# Manage pid_issuer
########################################################################

if [ "${PID_ISSUER}" == "0" ]
then
    echo
    echo -e "${SECTION}Manage pid_issuer${NC}"

    cd "${PID_ISSUER_DIR}"

    if [ "${STOP}" == "0" ]
    then
        echo -e "${INFO}Kill any running ${ORANGE}pid_issuer${NC}"
        killall pid_issuer || true
    fi
    if [ "${START}" == "0" ]
    then
        pushd "${WALLET_CORE_DIR}"
        echo -e "${INFO}Running pid_issuer database migrations${NC}"
        DATABASE_URL="postgres://${DB_USERNAME}:${DB_PASSWORD}@${DB_HOST}:5432/pid_issuer" cargo run --package wallet_server_migrations --bin wallet_server_migrations -- fresh
        popd

        echo -e "${INFO}Start ${ORANGE}pid_issuer${NC}"
        RUST_LOG=debug cargo run --package pid_issuer --no-default-features --features "postgres" --bin pid_issuer > "${TARGET_DIR}/pid_issuer.log" 2>&1 &

        echo -e "pid_issuer logs can be found at ${CYAN}${TARGET_DIR}/pid_issuer.log${NC}"
    fi
fi

########################################################################
# Manage verification_server
########################################################################

if [ "${VERIFICATION_SERVER}" == "0" ]
then
    # As part of the demo RP a verification_server is started
    echo
    echo -e "${SECTION}Manage verification_server${NC}"

    cd "${VERIFICATION_SERVER_DIR}"

    if [ "${STOP}" == "0" ]
    then
        echo -e "${INFO}Kill any running ${ORANGE}verification_server${NC}"
        killall verification_server || true
    fi
    if [ "${START}" == "0" ]
    then
        pushd "${WALLET_CORE_DIR}"
        echo -e "${INFO}Running verification_server database migrations${NC}"
        DATABASE_URL="postgres://${DB_USERNAME}:${DB_PASSWORD}@${DB_HOST}:5432/verification_server" cargo run --package wallet_server_migrations --bin wallet_server_migrations -- fresh
        popd

        echo -e "${INFO}Start ${ORANGE}verification_server${NC}"
        RUST_LOG=debug cargo run --package verification_server --no-default-features --features "allow_insecure_url,postgres" --bin verification_server > "${TARGET_DIR}/demo_rp_verification_server.log" 2>&1 &

        echo -e "verification_server logs can be found at ${CYAN}${TARGET_DIR}/demo_rp_verification_server.log${NC}"
    fi
fi

########################################################################
# Manage issuance_server
########################################################################

if [ "${ISSUANCE_SERVER}" == "0" ]
then
    # As part of the demo RP a issuance_server is started
    echo
    echo -e "${SECTION}Manage issuance_server${NC}"

    cd "${ISSUANCE_SERVER_DIR}"

    if [ "${STOP}" == "0" ]
    then
        echo -e "${INFO}Kill any running ${ORANGE}issuance_server${NC}"
        killall issuance_server || true
    fi
    if [ "${START}" == "0" ]
    then
        pushd "${WALLET_CORE_DIR}"
        echo -e "${INFO}Running issuance_server database migrations${NC}"
        DATABASE_URL="postgres://${DB_USERNAME}:${DB_PASSWORD}@${DB_HOST}:5432/issuance_server" cargo run --package wallet_server_migrations --bin wallet_server_migrations -- fresh
        popd

        echo -e "${INFO}Start ${ORANGE}issuance_server${NC}"
        RUST_LOG=debug cargo run --package issuance_server --no-default-features --features "allow_insecure_url,postgres" --bin issuance_server > "${TARGET_DIR}/demo_issuer_issuance_server.log" 2>&1 &

        echo -e "issuance_server logs can be found at ${CYAN}${TARGET_DIR}/demo_issuer_issuance_server.log${NC}"
    fi
fi

########################################################################
# Manage wallet_provider
########################################################################

if [ "${WALLET_PROVIDER}" == "0" ]
then
    echo
    echo -e "${SECTION}Manage wallet_provider${NC}"

    cd "${WP_DIR}"

    if [ "${STOP}" == "0" ]
    then
        echo -e "${INFO}Kill any running ${ORANGE}wallet_provider${NC}"
        killall wallet_provider || true
    fi
    if [ "${START}" == "0" ]
    then
        echo -e "${INFO}Running wallet_provider database migrations${NC}"
        pushd "${WALLET_CORE_DIR}"
        cargo run --package wallet_provider_migrations --bin wallet_provider_migrations -- fresh
        popd

        echo -e "${INFO}Start ${ORANGE}wallet_provider${NC}"
        RUST_LOG=debug cargo run --package wallet_provider --bin wallet_provider --features=android_emulator > "${TARGET_DIR}/wallet_provider.log" 2>&1 &

        echo -e "wallet_provider logs can be found at ${CYAN}${TARGET_DIR}/wallet_provider.log${NC}"
    fi
fi

########################################################################
# Manage configuration_server
########################################################################

if [ "${CONFIG_SERVER}" == "0" ]
then
    echo
    echo -e "${SECTION}Manage configuration_server${NC}"

    cd "${CS_DIR}"

    if [ "${STOP}" == "0" ]
    then
        echo -e "${INFO}Kill any running ${ORANGE}configuration_server${NC}"
        killall configuration_server || true
    fi
    if [ "${START}" == "0" ]
    then
        echo -e "${INFO}Start ${ORANGE}configuration_server${NC}"
        RUST_LOG=debug cargo run --package configuration_server --bin configuration_server > "${TARGET_DIR}/configuration_server.log" 2>&1 &

        echo -e "configuration_server logs can be found at ${CYAN}${TARGET_DIR}/configuration_server.log${NC}"
    fi
fi

########################################################################
# Manage update_policy_server

if [ "${UPDATE_POLICY_SERVER}" == "0" ]
then
    echo
    echo -e "${SECTION}Manage update_policy_server${NC}"

    cd "${UPS_DIR}"

    if [ "${STOP}" == "0" ]
    then
        echo -e "${INFO}Kill any running ${ORANGE}update_policy_server${NC}"
        killall update_policy_server || true
    fi
    if [ "${START}" == "0" ]
    then
        echo -e "${INFO}Start ${ORANGE}update_policy_server${NC}"
        RUST_LOG=debug cargo run --package update_policy_server --bin update_policy_server > "${TARGET_DIR}/update_policy_server.log" 2>&1 &

        echo -e "update_policy_server logs can be found at ${CYAN}${TARGET_DIR}/update_policy_server.log${NC}"
    fi
fi

########################################################################
# Manage brpproxy
########################################################################

if [ "${BRP_PROXY}" == "0" ]
then
    echo
    echo -e "${SECTION}Manage brpproxy${NC}"

    if [ "${STOP}" == "0" ]
    then
        echo -e "${INFO}Stopping ${ORANGE}brpproxy${NC}"
        docker compose --file "${DOCKER_COMPOSE_FILE}" down brpproxy || true
    fi
    if [ "${START}" == "0" ]
    then
        echo -e "Building and starting ${ORANGE}brpproxy${NC}"
        docker compose --file "${DOCKER_COMPOSE_FILE}" up --detach brpproxy
    fi
fi

########################################################################
# Manage gba_hc_converter
########################################################################

if [ "${GBA_HC}" == "0" ]
then
    echo
    echo -e "${SECTION}Manage gba_hc_converter${NC}"

    cd "${GBA_HC_CONVERTER_DIR}"

    if [ "${STOP}" == "0" ]
    then
        echo -e "${INFO}Stopping ${ORANGE}gba_hc_converter${NC}"
        killall gba_hc_converter || true
    fi
    if [ "${START}" == "0" ]
    then
        echo -e "Starting ${ORANGE}gba_hc_converter${NC}"

        encrypt_gba_v_responses
        RUST_LOG=debug cargo run --package gba_hc_converter --bin gba_hc_converter > "${TARGET_DIR}/gba_hc_converter.log" 2>&1 &

        echo -e "gba_hc_converter logs can be found at ${CYAN}${TARGET_DIR}/gba_hc_converter.log${NC}"
    fi
fi

########################################################################
# Manage wallet
########################################################################

if [ "${WALLET}" == "0" ]
then
    echo
    echo -e "${SECTION}Manage wallet${NC}"

    if [ "${START}" == "0" ]
    then
        cd "${BASE_DIR}"/wallet_app
        flutter run \
            --dart-define MOCK_REPOSITORIES=false \
            --dart-define ALLOW_INSECURE_URL=true \
            --dart-define UL_HOSTNAME="${UL_HOSTNAME:-}" \
            --dart-define SENTRY_DSN="${SENTRY_DSN:-}" \
            --dart-define SENTRY_ENVIRONMENT="${SENTRY_ENVIRONMENT}"
    fi
fi
