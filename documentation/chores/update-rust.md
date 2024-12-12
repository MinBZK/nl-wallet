# Howto: update Rust for NL Wallet

## Update Rust images

In the `nl-wallet-app-builder-dockerfiles` repository:

- In `rust.sh` update Rust to the desired version (see `nl-wallet-app-builder-dockerfiles` MR 60, as an example)
- Optional: update Rust-related dependencies in `rust.sh`, `rust_android.sh` and `cyclonedx.sh` to the desired version (beware for yanked versions)
- Get your MR approved and merged
- Start a pipeline with `IMAGE_NAMES` set to `rust flutter-rust android-flutter-rust cyclonedx`

## Update Rust macOS Runner

In the `macos-runner` repository:

- In `wallet.pkr.hcl` update Rust to the desired version (see `macos-runner` MR 4 as an example)
- Optional: update Rust-related dependencies too
- Don't forget to bump the version of the image (see `macos-runner` MR 5 as an example)
- Get your MR approved and merged
- Login on the macOS runner with SSH
- Get the latest version of the repo on the macOS runner. Note that the runner cannot access Gitlab, so you need to copy or push the repo to the runner manually. One approach is by adding a new remote to the repo and pushing the changes to the macOS runner (where `x.x.x.x` is the IP address of the macOS runner):

```
git add remote macos-runner ssh://x.x.x.x/~/sources/macos-runner
git push macos-runner
```

- Build the image with `packer build wallet.pkr.hcl` (see `macos-vm-image-templates/README.md` in the `macos-runner` repository) for more info)

## Update Rust workspace version

In the `nl-wallet` repository:

- Update all `Cargo.toml` files to the desired version (see commit `b72338fa25a4081678d7e5a0cf686ae6fa2f52c1`)
- Update all hashes in `.gitlab-ci.yml` and `deploy/gitlab/*.yml`
- Update the version of the macOS image in `.gitlab-ci.yml`, `deploy/gitlab/ios.yml` and `deploy/gitlab/rust.yml`
- Make sure the pipelines in your MR run successfully
- Get your MR approved and merged

And you're done! 🎉
