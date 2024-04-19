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
    ws, wallet_server:          Start the wallet_server.
    mrp, mock_relying_party:    Start the mock_relying_party.
    digid, digid_connector:     Start the digid_connector and a redis on docker.
    cs, configuration_server:   Start the configuration server
    brp:                        Start the Haal-Centraal BRP proxy with GBA mock.
    postgres:                   Start a PostgreSQL database, including pgadmin4, on docker.

  OPTION is any of:
    --all                       Start all of the above services.
    --default                   Start all of the above services, excluding docker and wallet.
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

MOCK_RELYING_PARTY=1
WALLET_PROVIDER=1
WALLET_SERVER=1
WALLET=1
DIGID_CONNECTOR=1
CONFIG_SERVER=1
BRP=1
POSTGRES=1

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
        ws|wallet_server)
            WALLET_SERVER=0
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
        cs|configuration_server)
            CONFIG_SERVER=0
            shift
            ;;
        brp)
            BRP=0
            shift
            ;;
        postgres)
            POSTGRES=0
            shift # past argument
            ;;
        --default)
            DIGID_CONNECTOR=0
            MOCK_RELYING_PARTY=0
            WALLET_SERVER=0
            WALLET_PROVIDER=0
            CONFIG_SERVER=0
            BRP=0
            shift # past argument
            ;;
        --all)
            DIGID_CONNECTOR=0
            POSTGRES=0
            MOCK_RELYING_PARTY=0
            WALLET_SERVER=0
            WALLET_PROVIDER=0
            WALLET=0
            CONFIG_SERVER=0
            BRP=0
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
        echo -e "Building and starting ${ORANGE}digid-connector${NC}"
        docker compose up --detach --build --force-recreate
    fi
fi

########################################################################
# Manage postgres

if [ "${POSTGRES}" == "0" ]
then
    echo
    echo -e "${SECTION}Manage postgres services${NC}"

    if [ "${STOP}" == "0" ]
    then
        echo -e "${INFO}Stopping postgres services${NC}"
        docker compose --file "${SCRIPTS_DIR}/docker-compose.yml" down postgresql pgadmin4 || true
    fi
    if [ "${START}" == "0" ]
    then
        echo -e "${INFO}Starting postgres services${NC}"
        docker compose --file "${SCRIPTS_DIR}/docker-compose.yml" up --detach postgresql pgadmin4
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
        RUST_LOG=debug cargo run --features "allow_http_return_url" --bin mock_relying_party > "${TARGET_DIR}/mock_relying_party.log" 2>&1 &

        echo -e "mock_relying_party logs can be found at ${CYAN}${TARGET_DIR}/mock_relying_party.log${NC}"
    fi
fi


########################################################################
# Manage wallet_server

if [ "${WALLET_SERVER}" == "0" ]
then
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
        echo -e "${INFO}Running wallet_server database migrations${NC}"
        pushd "${WALLET_CORE_DIR}"
        DATABASE_URL="postgres://${DB_USERNAME}:${DB_PASSWORD}@${DB_HOST}:5432/wallet_server" cargo run --bin wallet_server_migration -- fresh
        DATABASE_URL="postgres://${DB_USERNAME}:${DB_PASSWORD}@${DB_HOST}:5432/pid_issuer" cargo run --bin wallet_server_migration -- fresh
        popd
        echo -e "${INFO}Start ${ORANGE}wallet_server${NC}"
        RUST_LOG=debug cargo run --features "allow_http_return_url" --bin wallet_server \
                       > "${TARGET_DIR}/mrp_wallet_server.log" \
                       2>&1 &
        echo -e "wallet_server logs can be found at ${CYAN}${TARGET_DIR}/mrp_wallet_server.log${NC}"

        RUST_LOG=debug cargo run --features "allow_http_return_url,issuance" --bin wallet_server -- \
                       --config-file pid_issuer.toml \
                       --env-prefix pid_issuer \
                       > "${TARGET_DIR}/pid_issuer.log" \
                       2>&1 &
        echo -e "wallet_server logs can be found at ${CYAN}${TARGET_DIR}/pid_issuer.log${NC}"
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
        pushd "${WALLET_CORE_DIR}"
        cargo run --bin wallet_provider_migrations -- fresh
        popd
        echo -e "${INFO}Start ${ORANGE}wallet_provider${NC}"
        RUST_LOG=debug cargo run --bin wallet_provider > "${TARGET_DIR}/wallet_provider.log" 2>&1 &

        echo -e "wallet_provider logs can be found at ${CYAN}${TARGET_DIR}/wallet_provider.log${NC}"
    fi
fi

########################################################################
# Manage configuration_server

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
        RUST_LOG=debug cargo run --bin configuration_server > "${TARGET_DIR}/configuration_server.log" 2>&1 &

        echo -e "configuration_server logs can be found at ${CYAN}${TARGET_DIR}/configuration_server.log${NC}"
    fi
fi

########################################################################
# Manage brpproxy

if [ "${BRP}" == "0" ]
then
    echo
    echo -e "${SECTION}Manage brpproxy${NC}"

    if [ "${STOP}" == "0" ]
    then
        echo -e "${INFO}Stopping ${ORANGE}brpproxy${NC}"
        docker compose --file "${SCRIPTS_DIR}/docker-compose.yml" down brpproxy gbamock || true
    fi
    if [ "${START}" == "0" ]
    then
        echo -e "Building and starting ${ORANGE}brpproxy${NC}"
        docker compose --file "${SCRIPTS_DIR}/docker-compose.yml" up --detach brpproxy gbamock
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
        flutter run \
            --dart-define MOCK_REPOSITORIES=false \
            --dart-define ALLOW_HTTP_RETURN_URL=true \
            --dart-define ENV_CONFIGURATION=true \
            --dart-define UL_HOSTNAME="${UL_HOSTNAME:-}"
    fi
fi
