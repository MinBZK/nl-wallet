#!/bin/bash

set -e

export CONFIG_ENV=${CONFIG_ENV:-dev}

SCRIPTS_DIR="$(cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd)"
BINARY="$(dirname $SCRIPTS_DIR})/target/release/performance_test"

if [[ ${1:-} == '--skip-build' ]]; then
    if [[ ! -x $BINARY ]]; then
        >&2 echo "ERROR: No binary found: $BINARY"
        exit 1
    fi
    shift
else
    cargo build --manifest-path "${SCRIPTS_DIR}/Cargo.toml" \
        --release --bin performance_test \
        --features performance_test,allow_insecure_url
fi

NUM="${1:-1}"

START_DATE=$(date -u +%s)

pids=()
stop() {
    kill ${pids[@]}
}
trap stop INT

for ((i=1; i <= NUM; i++)); do
  (RUST_LOG=warn "$BINARY" 2>&1) & pids+=($!)
done

FAILED=0
for pid in ${pids[@]}; do
    if ! wait "$pid"; then
        FAILED=$((FAILED + 1))
    fi
done

END_DATE=$(date -u +%s)

echo "Load: $NUM"
echo "Duration: $(( END_DATE - START_DATE )) seconds"
if [[ $FAILED -gt 0 ]]; then
    echo "$FAILED out of $NUM failed"
    exit 1
fi
