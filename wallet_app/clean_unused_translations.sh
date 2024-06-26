#!/bin/bash
set -e # break on error
set -u # warn against undefined variables
set -o pipefail

dart run translations_cleaner clean-translations
