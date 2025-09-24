#!/usr/bin/env bash

set -euo pipefail

BASE_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"

fetch_original() {
  awk 'BEGIN { FS=" = " } /^'"$1"' / { print $2 }' "$BASE_DIR/gba_hc_converter.toml"
}

xml_path="$(fetch_original xml_path)"
encryption_key="$(fetch_original encryption_key)"
hmac_key="$(fetch_original hmac_key)"

SECRETS="$(kubectl get secret nl-wallet-gba-hc-converter-secrets -o yaml | yq -r '.data | map_values(@base64d)')"

cat > "$BASE_DIR/gba_hc_converter.toml" <<EOD
ip = "0.0.0.0"
port = 3012

[run_mode.all.gbav]
adhoc_url = "$(yq -r .adhoc_url <<< "$SECRETS")"
username = "$(yq -r .username <<< "$SECRETS")"
password = "$(yq -r .password <<< "$SECRETS")"
trust_anchor = "$(yq -r .trust_anchor <<< "$SECRETS")"

[run_mode.all.gbav.client_authentication]
certificate = "$(yq -r .client_cert <<< "$SECRETS")"
key = "$(yq -r .client_cert_key <<< "$SECRETS")"
chain = [
$(yq -r .client_cert_chain <<< "$SECRETS" | awk 'BEGIN { RS="," } { print "  \"" $1 "\"," }')
]

[run_mode.all.preloaded]
xml_path = $xml_path
encryption_key = $encryption_key
hmac_key = $hmac_key
EOD
