#!/usr/bin/env bash

set -euo pipefail

prompt="${1:?Specify the skill prompt, for example: /triage VULN-FINDINGS.json}"

cd "$(dirname "$0")"

exec env -i HOME="$HOME" PATH="$PATH" LANG=C.UTF-8 \
    CLAUDE_CODE_OAUTH_TOKEN="${CLAUDE_CODE_OAUTH_TOKEN:?}" \
    CLAUDE_CODE_MAX_TURNS="${CLAUDE_CODE_MAX_TURNS:?}" \
    claude -p "$prompt" \
    --model "${SCAN_MODEL:?}" \
    --max-budget-usd "${SCAN_MAX_BUDGET_USD:?}" \
    --permission-mode dontAsk \
    --add-dir ../../.. \
    --append-system-prompt "${NONINTERACTIVE:?}" \
    --verbose
