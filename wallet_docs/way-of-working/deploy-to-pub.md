# Deploying to PUB

A PUB rollout deploys the backend and apps from `nl-wallet`, plus the PUB
configuration from `nl-wallet-secrets` and `nl-wallet-static`. Deploy all
three repositories: changes to secrets or wallet configuration may already
be merged without being live on PUB, and the new `nl-wallet` version may
depend on them.

## Deployment checklist

1. **Bump the version and select a pipeline.** Follow
   [Version bump and release notes](#version-bump-and-release-notes).
2. **Deploy secrets and block old apps.** Complete the
   [nl-wallet-secrets steps](#deploy-secrets-and-block-old-apps).
3. **Deploy static files and wallet configuration.** Complete the
   [nl-wallet-static steps](#deploy-static-files-and-wallet-configuration).
4. **Deploy the backend.** Check whether the database changes require a
   [database reset](#database-resets), then run [**`deploy-apps-pub`**][backend].
   Wait for its child deployments and migrations to succeed.
5. **Build and upload the apps.** Run [**`build-pub-trigger`**][mobile]. Wait for
   `deploy-android-app-pub` and `deploy-ios-app-pub` in its child pipeline.
   CI uploads to Google Play **internal testing** and **TestFlight**;
   promotion happens in the store consoles.
6. **Verify before promotion.** Install both uploaded builds on physical
   devices. Check version, build number and PUB environment, and confirm
   that neither build is blocked by the update policy. Test registration,
   issuance, disclosure and changed functionality. Test updating an
   existing installation: release builds currently use a
   [database per Git commit][database-version], so a different commit starts
   a fresh wallet (PVW-4707).
   Record the tested build numbers and results.
7. **Promote those exact builds.** Open **NL Wallet (pub)** in both store
   consoles. In Google Play, promote Android from internal testing to the
   **Public Demo** closed testing track. In TestFlight, add the iOS build to
   the **Public Demo** external testing group. Wait for any required store
   review and release action to finish, then confirm the PUB audience can
   install both builds and check service health.

Use the **same `nl-wallet` main pipeline** for `deploy-apps-pub` and
`build-pub-trigger`. Check their **child pipelines**: a green parent does
not prove deployment succeeded.

## Version bump and release notes

The version MR below creates a new `nl-wallet` main pipeline. It copies the
latest merged PUB backend settings from [deploy-variables][variables].
The backend deployment uses that copy; `deploy-variables` has no separate
deployment job. The app build reads the settings again, so leave them
unchanged until the rollout is complete.

1. **Create the version MR.** In `nl-wallet`, choose a numeric `X.Y.Z` above
   versions already on PUB. Keep `-dev` for development builds and run
   [the version helper][version-helper]: `scripts/version.sh -s vX.Y.Z-dev`.
   Include all component version changes in the MR, targeting `main`.
2. **Update release notes in the same MR.** Rename the ongoing development
   draft to `wallet_docs/release-notes/vX.Y.Z.md`, change its heading to
   `# vX.Y.Z (dev)` and preserve its contents. After a formal release, start
   a new draft instead. The helper does not update release notes.
3. **Review and merge.** Confirm the component versions match and the release
   notes are included. Merge into `main` once the MR checks pass.
4. **Select the deployment pipeline.** Wait for the merged commit's `main`
   pipeline to pass. Use that same pipeline for `deploy-apps-pub` and
   `build-pub-trigger` in the deployment checklist.
5. **Change the version in Jira.**

Example: `0.7.1-dev` → `0.7.2-dev`, with `v0.7.1.md` → `v0.7.2.md` and heading
`# v0.7.2 (dev)`. The existing docs index picks up the renamed file.

A version change after verification requires rebuilding and testing again.

## Deploy secrets and block old apps

Work in `nl-wallet-secrets`. The PUB jobs below deploy all PUB ConfigMaps
and secrets, including changes merged for earlier features. They also
deploy the update policy that blocks old app versions.

1. **Set the minimum app version.** Create an MR that updates
   [`configmaps/pub/nl-wallet-update-policy-server.yaml`][policy-config]
   to block versions below the chosen minimum. Target `main`.
2. **Deploy the ConfigMaps and secrets.** Merge the MR and open its resulting
   `main` pipeline. Trigger [**`apply-configmaps-pub`**][policy-ci] and wait
   for it to succeed. The dependent **`apply-secrets-pub`** job runs
   automatically; wait for that job to succeed too.
3. **Wait for the policy server.** Policy changes automatically restart
   `update-policy-server`. Wait for the rollout to finish and confirm that
   the server is healthy. The server
   [loads its policy at startup][policy-server].
4. **Check an old app.** Force quit and reopen it to clear its one-hour
   policy cache. Confirm that it requires an update and that its update
   button opens the correct update destination. Complete this check before
   deploying the backend.

PUB users with blocked versions cannot use the wallet until the replacement
builds are available. This interruption is accepted for PUB rollouts.

Update policy compares the core's numeric version, ignoring `-dev` and build
numbers. For example, block versions below `0.7.2` with:

```toml
[update_policy]
"<0.7.2" = "Block"
```

Review existing rules too, so none blocks the new version.

Affected pods restart automatically when required.
`deploy-apps-pub` does not run these configuration jobs.

## Deploy static files and wallet configuration

Work in [nl-wallet-static][static-ci]. Use the same fresh `main` pipeline
for all steps below, so they publish the latest merged files and wallet
configuration.

1. **Deploy the static service.** Run `deploy-pub` and wait for it to succeed.
2. **Publish the static files.** Run `publish-assets-pub` and wait for it to
   succeed.
3. **Publish the wallet configuration.** Check that `wallet-config-pub`
   has succeeded. It signs the repository's `wallet-config-pub.json`.
   Run the manual `publish-config-pub` job to publish that signed
   configuration, and wait for it to succeed.

Complete these steps before deploying the backend or building the apps.
Existing apps also fetch the published wallet configuration. The scheduled
`republish-config-pub` job only renews the configuration already live on PUB;
it does not publish changes from the repository.

## Database resets

Normal schema migrations run automatically during backend deployment. If
this rollout requires a fresh database, reset only the affected databases
after old apps have been blocked. Use the matching manual
[`run-<service>-reset-pub` job][database-reset-jobs] in the `deploy-apps-pub`
child pipeline, for example `run-wallet-provider-reset-pub`. A reset deletes
the database's data and reapplies migrations.

The reset uses the currently deployed CronJob. Check that its migration
image matches the intended schema. Wait for the Kubernetes Job to complete
and check service health before continuing: a successful CI job only
confirms that the reset was started.

The separate `reset-pub` job wipes the broader environment and requires
redeployment.

Implementation checked 6 November 2026.

[version-helper]: SSSS/wallet/nl-wallet/-/blob/9e3f1833300bfa5584fc50c861eefd5fac06584b/scripts/version.sh
[backend]: SSSS/wallet/nl-wallet/-/blob/9e3f1833300bfa5584fc50c861eefd5fac06584b/deploy/gitlab/deploy-services.yml
[database-reset-jobs]: SSSS/wallet/nl-wallet/-/blob/9e3f1833300bfa5584fc50c861eefd5fac06584b/deploy/gitlab-templates/deploy-template.yml
[mobile]: SSSS/wallet/nl-wallet/-/blob/9e3f1833300bfa5584fc50c861eefd5fac06584b/deploy/gitlab/build-app-triggers.yml
[policy-config]: SSSS/wallet/nl-wallet-secrets/-/blob/a4ea0d675c538f8c06f5a7b916b8cba39c80b151/configmaps/pub/nl-wallet-update-policy-server.yaml
[policy-server]: SSSS/wallet/nl-wallet/-/blob/9e3f1833300bfa5584fc50c861eefd5fac06584b/wallet_core/update_policy/server/README.md
[variables]: SSSS/wallet/deploy-variables/-/tree/main/pub
[static-ci]: SSSS/wallet/nl-wallet-static/-/blob/d1d315aaec3aee0909ed37a85e3586d49f4418fc/.gitlab-ci.yml
[policy-ci]: SSSS/wallet/nl-wallet-secrets/-/blob/a4ea0d675c538f8c06f5a7b916b8cba39c80b151/.gitlab-ci.yml
[database-version]: SSSS/wallet/nl-wallet/-/blob/1e47b7061b84a59b32d076036f51472e0a3c1226/wallet_core/lib/utils/src/built_info.rs
