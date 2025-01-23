# How to update Rust (or anything really) for NL Wallet

The below chore documents how you can update Rust in the macos image we use. It
is mostly specifically about Rust, but the below guide has also succesfully been
used to upgrade Flutter for example (since essentially all changes you would
want to do are contained in the `wallet.pkr.hcl` file).

## Update Rust images

In the `nl-wallet-app-builder-dockerfiles` repository:

- In `rust-user.sh` update Rust to the desired version (see
  `nl-wallet-app-builder-dockerfiles` MR 60, as an example)
- Optional: update Rust-related dependencies in `rust-user.sh`, `cyclonedx.sh`
  to the desired version (beware for yanked versions)
- Run the pipeline manually and test out in your nl-wallet MR.
- Get your MR approved and merged, images will be created and uploaded to Harbor
  registry on merge to main. Update your MR with those images.

## Update Rust macOS Runner

In the `macos-runner` repository:

- In `wallet.pkr.hcl` update Rust to the desired version (see `macos-runner` MR
  4 as an example)
- Optional: update Rust-related dependencies too
- Don't forget to bump the version of the image (see `macos-runner` MR 5 as an
  example)
- Get your MR approved and merged
- Login on the macOS runner with SSH
- Get the latest version of the repo on the macOS runner. Note that the runner
  cannot access Gitlab, so you need to copy or push the repo to the runner
  manually. One approach is by adding a new remote to the repo and pushing the
  changes to the macOS runner (where `y` is the user and `x.x.x.x` is the IP
  address of the macOS runner):

```
git remote add macos-runner ssh://y@x.x.x.x/~/sources/macos-runner
git push macos-runner
```

Note: you might get an error related to the fact that the "remote" is not a bare
repository and so refuses to accept a push from you because the remote has that
specific (i.e., `main`) branch checked out. You can also use rsync to send your
data (warning: do **NOT** make any mistakes with path specification in the below
command, you could hose the entire user directory on the remote. **The trailing
slashes are important**):

```
cd to-parent-dir-containing-your-macos-runner-git-repo
rsync -av --delete -PHS your-local-macos-runner-git-repo-with-wanted-changes-in-main/ y@x.x.x.x/~/sources/macos-runner/
```

- Build the image with `packer build wallet.pkr.hcl` (see
  `macos-vm-image-templates/README.md` in the `macos-runner` repository) for
  more info)

## Update Rust workspace version

In the `nl-wallet` repository:

- Update all `Cargo.toml` files to the desired version (see commit
  `b72338fa25a4081678d7e5a0cf686ae6fa2f52c1`)
- Update the CI image in `.gitlab-ci.yml`
- Update the version of the macOS image in `.gitlab-ci.yml`,
  `deploy/gitlab/ios.yml` and `deploy/gitlab/rust.yml`
- Make sure the pipelines in your MR run successfully
- Get your MR approved and merged

And you're done! 🎉
