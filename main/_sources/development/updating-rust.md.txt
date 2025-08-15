# Updating Rust

The below chore documents how you can update Rust in the macos image we use. It
is mostly specifically about Rust, but the below guide has also succesfully been
used to upgrade Flutter for example (since essentially all changes you would
want to do are contained in the `flutter.sh` and `wallet.pkr.hcl` file).

## Docker image

- Update Rust for docker images to the desired version in:
  `deploy/docker-images/ci/rust-user.sh`
- Optional: update Rust-related dependencies in `rust-user.sh`, `cyclonedx.sh`
  to the desired version (beware for yanked versions)

## macOS image

- Update Rust for macOS image to the desired version in:
  `deploy/macos-image/wallet.pkr.hcl`
- Bump the version of the image
- Optional: update Rust-related dependencies too

## Build images

- Create commit and MR and run the build images jobs: `build-images-tag` and
  `macos-image-trigger`

## Use build images and update Rust workspace

- Update all `Cargo.toml` files to the desired version (see commit
  `1f0a26d1ac49947ed1da2abbc828d2f22ba7554f`)
- Change the image tags in `.gitlab-ci.yml` (`BUILD_TAG` and the `image` tag in
  `.env-macos-runner`)
- Commit and push again
- Get your MR approved and merged

And you're done! 🎉
