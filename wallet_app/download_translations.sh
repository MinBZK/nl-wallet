#!/bin/bash
set -e # break on error
set -u # warn against undefined variables
set -o pipefail

SCRIPTS_DIR=$(dirname "$(realpath "$(command -v "${BASH_SOURCE[0]}")")")

# Download 'nl-wallet-showcase-app.zip'
curl --request POST \
     --url https://api.lokalise.com/api2/projects/SSSS:branch/files/download \
     --header 'X-Api-Token: '"$LOKALISE_API_KEY"'' \
     --header 'accept: application/json' \
     --header 'content-type: application/json' \
     --data '
{
  "format": "arb",
  "replace_breaks": false,
  "plural_format": "icu",
  "compact": false,
  "original_filenames": false,
  "bundle_structure": "lib/l10n/intl_%LANG_ISO%.%FORMAT%",
  "exclude_tags": ["deprecated"]
}
' | grep -o '"bundle_url":"[^"]*' | grep -o '[^"]*$' | xargs wget -O "$SCRIPTS_DIR"/nl-wallet-showcase-app.zip

unzip -o "$SCRIPTS_DIR"/nl-wallet-showcase-app.zip -d "$SCRIPTS_DIR"

# Clean up
rm "$SCRIPTS_DIR"/nl-wallet-showcase-app.zip

# Generate new translation files
(cd "$SCRIPTS_DIR"; flutter gen-l10n)
