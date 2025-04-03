# How to update Rust (or anything really) for NL Wallet

The below chore documents how you can update Rust in the macos image we use. It
is mostly specifically about Rust, but the below guide has also succesfully been
used to upgrade Flutter for example (since essentially all changes you would
want to do are contained in the `wallet.pkr.hcl` file).

## Update Rust images

In the `nl-wallet-app-builder-dockerfiles` repository:

- In `rust-user.sh` update Rust to the desired version (see
  `nl-wallet-app-builder-dockerfiles` MR 60, as an example).
- Optional: update Rust-related dependencies in `rust-user.sh`, `cyclonedx.sh`
  to the desired version (beware for yanked versions).
- Run the pipeline manually and test out in your nl-wallet MR.
- Get your MR approved and merged, images will be created and uploaded to Harbor
  registry on merge to main.
- Update your MR with the new build image tag.

## Update Rust macOS Runner

In the `macos-runner` repository:

- In `wallet.pkr.hcl` update Rust to the desired version (see `macos-runner` MR
  4 as an example).
- Optional: update Rust-related dependencies too.
- Don't forget to bump the version of the image (see `macos-runner` MR 5 as an
  example).
- Get your MR approved and merged.
- Build the image via the CI pipeline.

## Update Rust workspace version

In the `nl-wallet` repository:

- Update all `Cargo.toml` files to the desired version (see commit
  `b72338fa25a4081678d7e5a0cf686ae6fa2f52c1`)
- Update the CI image in `.gitlab-ci.yml`
- Update the version of the macOS image in `.gitlab-ci.yml`
- Make sure the pipelines in your MR run successfully
- Get your MR approved and merged

And you're done! 🎉
