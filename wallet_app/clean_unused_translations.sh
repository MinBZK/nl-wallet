#!/bin/bash
set -e # break on error
set -u # warn against undefined variables
set -o pipefail

SCRIPTS_DIR=$(dirname "$(realpath "$(command -v "${BASH_SOURCE[0]}")")")

(cd "$SCRIPTS_DIR"; dart run translations_cleaner clean-translations)
