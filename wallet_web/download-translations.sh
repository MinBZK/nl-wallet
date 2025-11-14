#!/bin/bash
set -e # break on error
set -u # warn against undefined variables
set -o pipefail

SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"

# Download 'nl-wallet-web-locale.zip'
curl --request POST \
     --url https://api.lokalise.com/api2/projects/1574614866acd4406758d8.30660803:branch/files/download \
     --header "X-Api-Token: $LOKALISE_API_KEY" \
     --header 'accept: application/json' \
     --header 'content-type: application/json' \
     --data '
{
  "format": "json",
  "replace_breaks": false,
  "plural_format": "icu",
  "indentation": "2sp",
  "add_newline_eof": true,
  "original_filenames": false,
  "bundle_structure": "lib/l10n/%LANG_ISO%.%FORMAT%",
  "exclude_tags": ["deprecated"]
}
' | grep -o '"bundle_url":"[^"]*' | grep -o '[^"]*$' | xargs wget -O "$SCRIPTS_DIR"/nl-wallet-web-locale.zip

unzip -o "$SCRIPTS_DIR"/nl-wallet-web-locale.zip -d "$SCRIPTS_DIR"

# Clean up
rm "$SCRIPTS_DIR"/nl-wallet-web-locale.zip
