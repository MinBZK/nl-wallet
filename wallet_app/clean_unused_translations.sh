#!/bin/bash
set -e # break on error
set -u # warn against undefined variables
set -o pipefail

SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"

(cd "$SCRIPTS_DIR"; dart run translations_cleaner clean-translations)
