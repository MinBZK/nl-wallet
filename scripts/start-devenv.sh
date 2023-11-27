#!/usr/bin/env bash
# This script allows one to manage the individual services that are needed to run a completely local NL Wallet
# development environment.
#
# - nl-rdo-max-private (digid-connector)
#   This script requires this repo to exist in the same directory that contains the NL Wallet repo. Otherwise, customize
#   the DIGID_CONNECTOR_PATH environment variable in `scripts/.env`
# - mock_relying_party
# - pid_issuer
# - wallet_provider
# - wallet

set -e # break on error
set -u # warn against undefined variables
set -o pipefail
# set -x # echo statements before executing, useful while debugging

########################################################################
# Configuration
########################################################################

SCRIPTS_DIR=$(dirname "$(realpath "$(command -v "${BASH_SOURCE[0]}")")")
export SCRIPTS_DIR
BASE_DIR=$(dirname "${SCRIPTS_DIR}")
export BASE_DIR

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
    pi, pid_issuer:             Start the pid_issuer.
    mrp, mock_relying_party:    Start the mock_relying_party (this also starts a wallet_server).
    digid, digid_connector:     Start the digid_connector and a redis on docker.
    docker:                     Start a PostgreSQL database, including pgadmin4, on docker.

  OPTION is any of:
    --all                       Start all of the above services.
    --default                   Start all of the above services, excluding docker.
                                This option is provided when a PostgreSQL database is run and managed by the user.
    --stop                      Just stop all services
    -h, --help                  Show this help
"
}

########################################################################
# Check prerequisites

expect_command cargo "Missing binary 'cargo', please install the Rust toolchain"
expect_command docker "Missing binary 'docker', please install Docker (Desktop)"
expect_command flutter "Missing binary 'flutter', please install Flutter"

########################################################################
# Commandline arguments

PID_ISSUER=1
MOCK_RELYING_PARTY=1
WALLET_PROVIDER=1
WALLET=1
DIGID_CONNECTOR=1
DOCKER=1

USAGE=1

STOP=0
START=0

if [ "$#" == "0" ]
then
    USAGE=0
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
        pi|pid_issuer)
            PID_ISSUER=0
            shift # past argument
            ;;
        mrp|mock_relying_party)
            MOCK_RELYING_PARTY=0
            shift # past argument
            ;;
        digid|digid_connector)
            DIGID_CONNECTOR=0
            shift # past argument
            ;;
        docker)
            DOCKER=0
            shift # past argument
            ;;
        --default)
            DIGID_CONNECTOR=0
            PID_ISSUER=0
            MOCK_RELYING_PARTY=0
            WALLET_PROVIDER=0
            WALLET=0
            shift # past argument
            ;;
        --all)
            DIGID_CONNECTOR=0
            DOCKER=0
            MOCK_RELYING_PARTY=0
            PID_ISSUER=0
            WALLET_PROVIDER=0
            WALLET=0
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
            echo -e "${RED}ERROR${NC}: Unknown argument: $1"
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
        echo -e "Building ${ORANGE}digid-connector${NC} image"
        docker compose build
        echo -e "Starting ${ORANGE}digid-connector${NC}"
        docker compose up -d
    fi
fi

########################################################################
# Manage docker

if [ "${DOCKER}" == "0" ]
then
    echo
    echo -e "${SECTION}Manage wallet docker services${NC}"

    cd "${BASE_DIR}"

    if [ "${STOP}" == "0" ]
    then
        echo -e "${INFO}Stopping docker services${NC}"
        docker compose down || true
    fi
    if [ "${START}" == "0" ]
    then
        echo -e "${INFO}Starting docker services${NC}"
        docker compose up -d
    fi
fi

########################################################################
# Manage pid_issuer

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
        echo -e "${INFO}Start ${ORANGE}pid_issuer${NC}"
        RUST_LOG=debug cargo run --bin pid_issuer --features disable_tls_validation > "${TARGET_DIR}/pid_issuer.log" 2>&1 &

        echo -e "pid_issuer logs can be found at ${CYAN}${TARGET_DIR}/pid_issuer.log${NC}"
    fi
fi

########################################################################
# Manage mock_relying_party

if [ "${MOCK_RELYING_PARTY}" == "0" ]
then
    echo
    echo -e "${SECTION}Manage mock_relying_party${NC}"

    cd "${MOCK_RELYING_PARTY_DIR}"

    if [ "${STOP}" == "0" ]
    then
        echo -e "${INFO}Kill any running ${ORANGE}mock_relying_party${NC}"
        killall mock_relying_party || true
    fi
    if [ "${START}" == "0" ]
    then
        echo -e "${INFO}Start ${ORANGE}mock_relying_party${NC}"
        RUST_LOG=debug cargo run --bin mock_relying_party > "${TARGET_DIR}/mock_relying_party.log" 2>&1 &

        echo -e "mock_relying_party logs can be found at ${CYAN}${TARGET_DIR}/mock_relying_party.log${NC}"
    fi

    # As part of the MRP a wallet_server is started
    echo
    echo -e "${SECTION}Manage wallet_server${NC}"

    cd "${MRP_WALLET_SERVER_DIR}"

    if [ "${STOP}" == "0" ]
    then
        echo -e "${INFO}Kill any running ${ORANGE}wallet_server${NC}"
        killall wallet_server || true
    fi
    if [ "${START}" == "0" ]
    then
        echo -e "${INFO}Start ${ORANGE}wallet_server${NC}"
        RUST_LOG=debug cargo run --bin wallet_server > "${TARGET_DIR}/mrp_wallet_server.log" 2>&1 &

        echo -e "wallet_server logs can be found at ${CYAN}${TARGET_DIR}/mrp_wallet_server.log${NC}"
    fi
fi


########################################################################
# Manage wallet_provider

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
        pushd ${WALLET_CORE_DIR}
        cargo run --bin wallet_provider_migrations -- fresh
        popd
        echo -e "${INFO}Start wallet_provider${NC}"
        RUST_LOG=debug cargo run --bin wallet_provider > "${TARGET_DIR}/wallet_provider.log" 2>&1 &

        echo -e "wallet_provider logs can be found at ${CYAN}${TARGET_DIR}/wallet_provider.log${NC}"
    fi
fi

########################################################################
# Manage wallet

if [ "${WALLET}" == "0" ]
then
    echo
    echo -e "${SECTION}Manage wallet${NC}"

    if [ "${START}" == "0" ]
    then
        cd "${BASE_DIR}"/wallet_app
        flutter run --dart-define MOCK_REPOSITORIES=false --dart-define DISABLE_TLS_VALIDATION=true --dart-define ENV_CONFIGURATION=true
    fi
fi
