#!/bin/bash
set -e # break on error
set -u # warn against undefined variables
set -o pipefail

SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"

# Not a dev_dependency: translations_cleaner 0.2.1 requires analyzer < 11, which conflicts with freezed 4
dart pub global activate translations_cleaner 0.2.1
(cd "$SCRIPTS_DIR"; dart pub global run translations_cleaner clean-translations)
