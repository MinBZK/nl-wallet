#!/usr/bin/env bash
set -euo pipefail

# This script:
#  - Cleans the rust_core/target folder.
#  - Resets the devenv and runs `cargo check` to populate the Rust cache, so that
#    this is already done for the next time `cargo` commands are run.

usage() {
    cat <<EOF
Usage: $0 [-P|--skip-postgres] [-h|--help]

Options:
  -P, --skip-postgres    Skip starting Postgres; assumes it is already running
  -h, --help             Show this help message
EOF
}

START_POSTGRES=1

while [[ $# -gt 0 ]]; do
    case "$1" in
        -P | --skip-postgres)
            START_POSTGRES=0
            shift
            ;;
        -h | --help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown argument: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"
BASE_DIR="$(dirname -- "${SCRIPTS_DIR}")"

# Clear rust_core/target
cargo clean --manifest-path "${BASE_DIR}/wallet_core/Cargo.toml"

# Some devenv files live in wallet_core/target, so we need to rerun setup-devenv.sh
SKIP_DIGID_CONNECTOR=1 SKIP_WALLET_WEB=1 bash "${BASE_DIR}/scripts/setup-devenv.sh"

# Also reset the DB
if [[ "${START_POSTGRES}" == "1" ]]; then
    bash "${SCRIPTS_DIR}/start-devenv.sh" postgres
fi
bash "${SCRIPTS_DIR}/migrate-db.sh" fresh

# Run cargo check to populate the cache
cargo check --manifest-path "${BASE_DIR}/wallet_core/Cargo.toml" --all-features --all-targets --tests

# Clear old test databases. Suppress output, we don't care if the container does not run or exist
docker container stop wallet-postgres-test > /dev/null 2>&1 || true
docker container rm wallet-postgres-test > /dev/null 2>&1 || true
