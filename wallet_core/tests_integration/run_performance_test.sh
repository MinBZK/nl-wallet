#!/bin/bash

set -e

SCRIPTS_DIR="$(cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd)"
NUM="${1:-1}"

export CONFIG_ENV=${CONFIG_ENV:dev}

BINARY=${SCRIPTS_DIR}"/../target/release/performance_test"
cargo build --manifest-path "${SCRIPTS_DIR}/Cargo.toml" \
    --release --bin performance_test \
    --features performance_test,allow_insecure_url

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
