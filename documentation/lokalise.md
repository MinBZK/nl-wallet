# Lokalise

Internally, this project uses the commercial Lokalise service to manage translations. This service is currently not accessible for external contributors. The following is for contributors with access.

## Workflow

The most common workflow to add or edit translations is to go to the Lokalise [nl-wallet-app](https://app.lokalise.com/project/SSSS/) project and edit the translations in the web UI. After doing so you can export the new translations as .arb files to be used in the project.

### Exporting translations (lokalise -> app)

To download the latest translations from Lokalise, go to the 'Download' tab and select the following:

- File format: Flutter (.arb)
- Include all platform keys (no idea if this is necessary)
- Languages: All
- File structure:
  - One file per language
  - `intl_%LANG_ISO%.%FORMAT%`
- Advanced settings:
  - Disable `compact format`
  - Enable `include description`
  - Enable `Replace line breaks with \n`
- App triggers:
  - None

Click `Preview` to preview your changes, and click `Build and download` to download the translations.
Place the contents of the download zip file in the `wallet_app/lib/l10n` folder.
Al

#### Export script

Alternatively you can run the following script (inside the `wallet_app` dir), make sure to set `LOKALISE_API_KEY`, an API Token can be generated from the Lokalise dashboard at Profile Settings -> API Tokens.

```bash
#!/bin/bash

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
  "bundle_structure": "lib/l10n/intl_%LANG_ISO%.%FORMAT%"
}
' | grep -o '"bundle_url":"[^"]*' | grep -o '[^"]*$' | xargs wget

# Unzip 'nl-wallet-showcase-app.zip'
unzip -o nl-wallet-showcase-app.zip

# Clean up
rm nl-wallet-showcase-app.zip

# Generate new translation files
flutter gen-l10n
```

### Importing keys/translations (app -> lokalise)

When managing the translations in Lokalise, importing from the app should in theory never be necessary, however if the translations do run out of sync (e.g. because keys were only added in the app repo) the following can be done to sync the new keys back to lokalise.

Go to the upload tab of the nl-wallet-app project and use the following options:

- [x] Replace \n with line break
- [x] Convert to universal placeholders
- [x] Detect ICU plurals

- [ ] Tag keys
- [ ] Differentiate keys by file
- [ ] Fill empty keys with key name
- [ ] Hide from contributors
- [ ] Pre-translate with 100% TM matches
- [x] Replace modified values (disable to only push new keys)
- [ ] Enable cleanup mode  (*Caution*: Can be useful to clean up Lokalise, but remember to create a snapshot first! This can be done via: More -> Snapshots -> Take new snapshot)

After uploading both the intl_en.arb and intl_nl.arb and processing them the localisations should be in sync with the repository.

#### Import script

Alternatively you can run the following script (inside the `wallet_app` dir), make sure to set `LOKALISE_API_KEY`, an API Token can be generated from the Lokalise dashboard at Profile Settings -> API Tokens.

Note that the script always creates a snapshot before uploading and that `replace_modified` is disabled by default. Run the script with the `--replace-modified` option to enable this.

```bash
#!/bin/bash

# Check for the --replace-modified flag
replaceModified="false"
for arg in "$@"; do
  case $arg in
  --replace-modified)
    echo "Uploading with replace_modified enabled"
    replaceModified="true"
    ;;
  esac
done

# Create a snapshot for safety purposes
echo "Creating snapshot..."
curl --request POST \
  --url https://api.lokalise.com/api2/projects/SSSS/snapshots \
  --header 'X-Api-Token: '"$LOKALISE_API_KEY"'' \
  --header 'accept: application/json' \
  --header 'content-type: application/json' \
  --data '
{
  "title": "Upload script"
}
'
echo ""

# Upload language files
supportedLocales=("en" "nl")
for languageIso in "${supportedLocales[@]}"; do
  languageFile="lib/l10n/intl_$languageIso.arb"
  fileContents=$(base64 <"$languageFile")
  echo "Uploading ${languageFile}..."
  curl --request POST \
    --url https://api.lokalise.com/api2/projects/SSSS:branch/files/upload \
    --header 'X-Api-Token: '"$LOKALISE_API_KEY"'' \
    --header 'accept: application/json' \
    --header 'content-type: application/json' \
    --data '
    {
      "data": "'"$fileContents"'",
      "filename": "'"$languageFile"'",
      "lang_iso": "'"$languageIso"'",
      "convert_placeholders": true,
      "detect_icu_plurals": true,
      "slashn_to_linebreak": true,
      "replace_modified": '"$replaceModified"'
    }
    '
  echo ""
done

```
