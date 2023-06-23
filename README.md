# EDI - NL Public Reference Wallet

Within the European Digital Identity program of the Ministry of the Interior and Kingdom Relations,
a team is working on a Dutch Public Reference Wallet: the NL Wallet. The program conducts its work
in an open and transparent way. This means that the products of the development team – the app
designs and source code – are viewable for everyone who wants to. It’s also possible to contribute
to the project. We offer three platforms for contributions: [Pleio](https://edi.pleio.nl) as a
central hub, [GitHub](https://github.com/MinBZK/nl-wallet-demo-app) for the source code
and [Figma](https://www.figma.com/file/dO5pKIIIyDgG0N2ZX4C2xd/2212_V1_Designs_NL-Voorbeeld-Wallet?node-id=1%3A3717&t=nwzAEOJZJEb0eC0o-1)
for screens/flows.

The development of a public and open source reference wallet is a dynamic process. What has been
released in the current version, may in a next version be developed further, be modified drastically
or even removed altogether.

We look forward to
your [feedback and great ideas](mailto:edi@minbzk.nl?subject=Contribution%20via%20GitHub).

### Managing expectations

It’s important to note that a demo app will always be developed ahead of an actually functioning
wallet. A clickable demo app contains little technical implementation under the hood. That makes it
fast and easy to explore, visualize and discuss different scenarios and possibilities. It also
provides a testbed for engineers to explore / experiment / try out certain technical aspects of a
wallet.

When you look at the screens
in [Figma](https://www.figma.com/file/dO5pKIIIyDgG0N2ZX4C2xd/2212_V1_Designs_NL-Voorbeeld-Wallet?node-id=1%3A3717&t=nwzAEOJZJEb0eC0o-1)
or in the demo app, they suggest a lot of functionality that seems to work, but which still requires
a lot of development to realize in practice.

It’s possible to use such a wallet in a lot of different situations, for example to share your
diploma’s when applying for a job, or to prove that you’re 18+ to buy a beer. With the demo app, we
can demonstrate how such a wallet would work in a relatively simple way.

Although parts of this demo might later be used in wallet related projects, this demo is by no means
intended to be production quality code. The goal is to release the first working version of the
public reference wallet at the end of 2023 and to make it available in phases for various services.

# Table of contents

- [Current release](#current-release)
- [Documentation](#documentation)
- [Licensing](#licensing)
- [Contributing](#contributing)
- [Getting started](#getting-started)
    * [Setup development environment](#setup-development-environment)
- [File structure](#file-structure)
    * [Code](#code)
- [Conventions](#conventions)
    * [Git](#git)
- [Distribution](#distribution)
- [Troubleshooting](#troubleshooting)

# Current release

See the [releases page](https://github.com/MinBZK/nl-wallet-demo-app/releases) for the latest
release. The current releases are a clickable demo app that show a number of 'flows' that the user
of a wallet can walk through in the context of example use cases. These try out use cases have been
chosen because of a variety of reasons: what we want to show, what we want to learn, if they are
realistic use cases in practice, and if it is feasible to prototype them in this phase. You can
follow the latest work by subscribing to the releases of this GitHub repository at the top of this
page.

# Documentation

**TL;DR**

With the NL reference wallet we want to achieve the following things:

- We want to validate the feasibility of the framework as proposed in the EU.
- We want to explore how we can set the bar in terms of privacy protection, security, usability and
  inclusion.
- We want to learn what this development means for citizens, businesses, other governments and
  public service providers.
- We want to help citizens, especially those with special needs, in the best way possible.
- We want to offer a testing ground for a variety of use cases.
- We want to share the lessons we learn with the public and share them with the EU.

If you want to learn more about the NL Wallet development, please read the background information on
the Pleio hub. The development of the user flows and screens can be followed
through [Figma](https://www.figma.com/file/dO5pKIIIyDgG0N2ZX4C2xd/2212_V1_Designs_NL-Voorbeeld-Wallet?node-id=1%3A3717&t=nwzAEOJZJEb0eC0o-1)
.

# Licensing

The source code of the NL Wallet is released under the [EUPL license](./LICENSES/EUPL-1.2.txt). The
documentation is released under the [CC0 license](./LICENSES/CC0-1.0.txt). Please see
the [.reuse/dep5](./.reuse/dep5) file for more details, which follows
the [Reuse specfication](https://reuse.software/spec/).

# Contributing

We’re releasing the source code with the explicit intention of allowing contributions. The
coordination of the project lies with the development team of the European Digital Identity Progam,
but we’re open to all contributions. You can directly create a new Pull Request via Github,or
contact the community manager
via [edi@minbzk.nl](mailto:edi@minbzk.nl?subject=Contribution%20via%20GitHub).

The development team works on the repository in a private fork (for reasons of compliance with
existing processes) and shares its work as often as possible. If you watch the repository on GitHub,
you will be notified of a new release. We will also send a notification through Pleio.

Although we are open to contributions, please consider the nature of this project as outlined in
this Readme. At this stage the most useful way to contribute to the project is to participate on our
community site [edi.pleio.nl](https://edi.pleio.nl), and visit
our [EDI Meet-ups and/or Heartbeats](https://edi.pleio.nl/events).

If you plan to make non-trivial changes, we recommend that you open an issue beforehand where we can
discuss your planned changes. This increases the chance that we might be able to use your
contribution (or it avoids doing work if there are reasons why we wouldn't be able to use it).

Note that all commits should be signed using a GPG key.

# Getting started

This section contains the general setup requirements of the project. For more details on configuration of the [wallet app](./wallet_app/README.md), the [wallet core](./wallet_core/README.md) and the [wallet_provider](./wallet_core/wallet_provider/README.md), please see the corresponding README files.

## Setup development environment

The app's UI is build using Flutter, but to avoid tying the app to Flutter & Dart, all core business
logic is build using Rust. This gives us the more flexibility to migrate to completely native
iOS/Android app's if the need arises. This does mean building the app is slightly more complex than
a simple `flutter run`. This section describes how to set up your environment.

### Requirements:

- Flutter
- Rust (incl. additional targets)
- Android SDK + NDK (for Android builds)
- Xcode (for iOS builds)

#### Flutter

To install Flutter follow this [installation guide](https://flutter.dev/docs/get-started/install).
You can validate your initial setup by running `flutter doctor`.

**Easily manage your local Flutter version using: Flutter Version Manager (FVM)**
FVM is a simple CLI to manage Flutter SDK versions per project. It enables fast switching between
Flutter versions and pin them to your Flutter project. When using FVM; all Flutter related
command need to be prefixed with `fvm`, e.g. `fvm flutter run`.

_Optional step:_
To install FVM follow this [installation guide](https://fvm.app/docs/getting_started/installation).
You can validate your initial setup by running `fvm flutter doctor` after the installations. Hit [Y]es when asked to install the pinned Flutter version defined in [fvm_config.json](wallet_app/.fvm/fvm_config.json).

Note that FVM only pins the Flutter version for local development, not the CI pipelines.

#### Rust

To install Rust & Cargo (the package manager) follow
the [installation guide](https://www.rust-lang.org/tools/install). After installing rust make sure
to add the following targets:

- For iOS: `rustup target add aarch64-apple-ios x86_64-apple-ios`
- For Android: `rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`

#### Android

To build for android you need to have the Android SDK and NDK installed on your system. Likely the
easiest way to do so is:

1. Install [Android Studio](https://developer.android.com/studio) (includes SDK)
2. Open Android Studio
    1. Tools -> SDK Manager
    2. Select 'SDK Tools' Tab
    3. Check latest 'NDK (Side by side)' in the list (>= v25.2.9519653)
    4. Hit 'apply' to install
3. Add `export ANDROID_HOME="$HOME/Library/Android/sdk"` to your `~/.bash_profile` or
   `~/.zshrc`, this will make sure the Android SDK is available in your path and automatically
   picks up the NDK version you installed in the previous step
4. Tell gradle where it can find the NDK by updating one of your `gradle.properties`,
   e.g. `echo "NDK_HOME=\"$HOME/Library/Android/sdk/ndk/{ndk_version}\"" >> ~/.gradle/gradle.properties`
5. Run `cargo install cargo-ndk` (>= v3.0.0) to be able to build the Rust code for Android
6. Optional: let Android Studio use Flutter SDK installed via FVM by following
   [these steps](https://fvm.app/docs/getting_started/configuration#android-studio)

#### iOS

Install [Xcode](https://apps.apple.com/us/app/xcode/id497799835?mt=12)

### Validate

After doing the above `flutter doctor` should report that at least the following are installed
successfully:

- Flutter
- Android toolchain
- Xcode

You should now be able to launch an Android Emulator or iOS Simulator and run the app by following these steps:
- `cd wallet_app`
- `flutter pub get`
- `flutter run`! 🎉

# File structure

## Code

All `Dart` code goes in the `wallet_app/lib/` directory and their appropriate sub-directories.

All `Rust` code goes in the `wallet_core/` directory and their appropriate sub-directories.

### Flutter <-> Rust Bridge
Communication between the Flutter and Rust layers relies on the `flutter_rust_bridge` package, the bridge code is generated. The definition of this bridge can is located at `/wallet_core/src/api.rs` and generation is done with the following command:

```
cargo run --manifest-path wallet_core/flutter_rust_bridge_codegen/Cargo.toml
```

The generated code is currently checked in, so that generation only has to be performed when the API changes.


# Conventions

## Git

### Commit message

- Capitalize the subject line
- Use the imperative mood in the subject line
- Do not end the subject line with a period
- Wrap lines at 72 characters

## Branch names

- Prefix the branch name with the Jira code for the story or subtask the branch relates to.
  If there is no story or subtask, strongly consider making one or forego the prefix.
- The rest of the branch name should be a short description of the purpose of the branch, in lowercase and separated by hyphens.
  The description should be clear enough that any reader should understand it without having to look up the Jira ticket.
  Consider starting the description with the component that is being worked on, e.g. `ci-` or `core-`.

Example of a branch name: **PVW-123-wp-teapot-status-code**

### PR title

See [commit message](#commit-message).

### PR merge

- Default to squash merge (combined with PR title conventions)

# Distribution

Follow these steps to (force) distribute internal `alpha` & `beta` builds that target the Android
platform":

### Alpha

> Use `Alpha` distribution at any time during development cycle.

* Push commit of your choosing to: alpha/v{X.Y.Z}
* After the GitHub Action has completed successfully; install the release via F-Droid repo

### Beta

> Use `Beta` distribution at the end of a sprint cycle; to represent the sprint demo version.

* `$ git fetch && git pull`
* Push `main` branch to: beta/v{X.Y.Z}
* After the GitHub Action has completed successfully; install the release via F-Droid repo

# Troubleshooting

### Initial checkout / branch switch

Generate/update localisation files (to compile/run the project successfully):

    $ flutter gen-l10n
