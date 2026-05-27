#!/usr/bin/env bash

set -euo pipefail

SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"
source "${SCRIPTS_DIR}/run-demo.env"

export ENTRYPOINT_SCRIPT="${BASH_SOURCE[0]}"
export ANDROID_ENV_NAME="${TARGET_ENV_NAME}"
export ANDROID_ENV_SLUG="${TARGET_ENV_SLUG}"
export PIPELINE_SOURCE
export PIPELINE_DESCRIPTION
export APPLICATION_ID
export APP_NAME
export UL_HOSTNAME

exec "${SCRIPTS_DIR}/run-android-env.sh" "$@"
