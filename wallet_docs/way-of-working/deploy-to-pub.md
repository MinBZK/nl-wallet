# Deploying to PUB

PUB rollouts are complete deployments: bump the version, block old app
versions, deploy the backend, then build, verify and promote both Android
and iOS apps.

## Deployment checklist

1. **Bump the version.** Follow
   [Version bump and release notes](#version-bump-and-release-notes).
2. **Block old app versions.** Follow
   [Update-policy deployment](#update-policy-deployment) and verify that an
   old app is blocked before deploying the backend. PUB users may be unable
   to use the wallet until the replacement builds are available; this
   interruption is accepted for PUB rollouts.
3. **Deploy the backend.** Run [**`deploy-apps-pub`**][backend]. Wait for its
   child deployments and migrations to succeed.
4. **Build and upload the apps.** If this rollout changes the signed wallet
   configuration, publish it first (see
   [Configuration, when needed](#configuration-when-needed)). Then run
   [**`build-pub-trigger`**][mobile]. Wait for
   `deploy-android-app-pub` and `deploy-ios-app-pub` in its child pipeline.
   CI uploads to Google Play **internal testing** and **TestFlight**;
   promotion happens in the store consoles.
5. **Verify before promotion.** Install both uploaded builds on physical
   devices. Check version, build number and PUB environment; test registration,
   issuance, disclosure and changed functionality. Test updating an existing
   installation: release builds currently use a [database per Git commit][database-version],
   so a different commit starts a fresh wallet (PVW-4707).
   Record the tested build numbers and results.
6. **Promote those exact builds.** Open **NL Wallet (pub)** in both store
   consoles. In Google Play, promote Android from internal testing to the
   **Public Demo** closed testing track. In TestFlight, add the iOS build to
   the **Public Demo** external testing group. Wait for any required store
   review and release action to finish, then confirm the PUB audience can
   install both builds and check service health.

Use the **same main pipeline** for both triggers and check their **child
pipelines**: a green parent does not prove deployment succeeded.

## Version bump and release notes

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

## Update-policy deployment

1. **Create an MR.** In `nl-wallet-secrets`, update
   [`configmaps/pub/nl-wallet-update-policy-server.yaml`][policy-config]
   to block versions below the chosen minimum. Target `main`.
2. **Deploy the policy.** Merge the MR and, in its resulting `main` pipeline,
   trigger [**`apply-configmaps-pub`**][policy-ci]. Wait for it and the
   automatically triggered **`apply-secrets-pub`** to succeed. Then have the
   PUB operator restart `update-policy-server` and wait for it to be healthy:
   these jobs do not restart it, and the [policy loads at startup][policy-server].
3. **Check the result.** Fully restart an old app to bypass the one-hour
   policy cache. Before deploying the backend, verify that it requires an
   update and opens the correct update destination. During app verification,
   fully restart each new build and confirm that it remains usable.

Update policy compares the core's numeric version, ignoring `-dev` and build
numbers. For example, block versions below `0.7.2` with:

```toml
[update_policy]
"<0.7.2" = "Block"
```

Review existing rules too, so none blocks the new version.

The configuration job also applies other ConfigMaps and recreates GBA test
data; its dependent job applies PUB secrets. `deploy-apps-pub` does not apply
the policy ConfigMap.

## Configuration, when needed

- **Backend values:** merge changes in [deploy-variables][variables] before
  creating the selected pipeline, or regenerate its deployment artifacts.
  The mobile child fetches values again later; keep them compatible.
- **Signed wallet configuration:** update `wallet-config-pub.json` in
  [nl-wallet-static][static-ci], then run `wallet-config-pub` and the manual
  `publish-config-pub` before building apps. Use a freshly signed artifact.
  Existing apps also fetch this configuration. Scheduled `republish-config-pub`
  only renews the live configuration; it does not publish repository changes.

Implementation checked 6 November 2026.

[version-helper]: SSSS/wallet/nl-wallet/-/blob/9e3f1833300bfa5584fc50c861eefd5fac06584b/scripts/version.sh
[backend]: SSSS/wallet/nl-wallet/-/blob/9e3f1833300bfa5584fc50c861eefd5fac06584b/deploy/gitlab/deploy-services.yml
[mobile]: SSSS/wallet/nl-wallet/-/blob/9e3f1833300bfa5584fc50c861eefd5fac06584b/deploy/gitlab/build-app-triggers.yml
[policy-config]: SSSS/wallet/nl-wallet-secrets/-/blob/a4ea0d675c538f8c06f5a7b916b8cba39c80b151/configmaps/pub/nl-wallet-update-policy-server.yaml
[policy-server]: SSSS/wallet/nl-wallet/-/blob/9e3f1833300bfa5584fc50c861eefd5fac06584b/wallet_core/update_policy/server/README.md
[variables]: SSSS/wallet/deploy-variables/-/tree/main/pub
[static-ci]: SSSS/wallet/nl-wallet-static/-/blob/d1d315aaec3aee0909ed37a85e3586d49f4418fc/.gitlab-ci.yml
[policy-ci]: SSSS/wallet/nl-wallet-secrets/-/blob/a4ea0d675c538f8c06f5a7b916b8cba39c80b151/.gitlab-ci.yml
[database-version]: SSSS/wallet/nl-wallet/-/blob/1e47b7061b84a59b32d076036f51472e0a3c1226/wallet_core/lib/utils/src/built_info.rs#L40
