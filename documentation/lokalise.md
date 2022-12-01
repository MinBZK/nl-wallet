# Lokalise

Internally, this project uses the commercial Lokalise service to manage translations. This service is currently not accessible for external contributors. The following is for contributors with access.

## Importing keys

Keys can be imported from the app repository on the GitHub integration page of the project. You can find this page in the project via Apps -> GitHub -> Manage with sufficient access.

In `Pull options`; Enable both: `Detect ICU plurals` & `Replace \n with line break`.

Click on the `Pull now` button to import the keys. You can also view the logs to see if everything went successfully. Be mindful that at the moment only the keys and translations of the primary language (which is set to English now) are imported.

> Note; the above is still a manual action, but it would be nice to automatically import new keys with a commit hook. This is quite easy to do, but just hasn’t been done yet.

## Exporting translations

To create a `Pull Request` with all your edited translations, go to the Download page of the project. All settings should come prefilled after first use. Use the following settings:

- File format: Flutter (.arb)
- Include all platform keys (no idea if this is necessary)
- File structure:
   - One file per language
   - `lib/src/localization/app_%LANG_ISO%.%FORMAT%`
- Advanced settings:
   - Disable `compact format`
   - Enable `include description`
   - Enable `numeric plural forms` under `Plural format`
- App triggers:
   - GitHub
   - Filter repositories: enter the name of the repository here
   - Commit message: `localization: Update translations`

Click `Preview` to preview your changes, and click `Build only` to create the PR.