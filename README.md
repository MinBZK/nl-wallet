# EDI - NL Public Reference Wallet

Under the [Working Agenda Value Driven Digitization][1], the Dutch government is
preparing for the introduction of European digital identity wallets (in short
ID-wallets) through the revision of the [eIDAS-regulation][2]. One of the ways
in which they are doing this, is by developing a public reference wallet called
the NL Wallet. These ID-wallets will be mobile apps that citizens can use to
identify (or ‘log in’) to public and private online services, share data about
themselves, and sign electronically.

The first version of the NL Wallet will focus on online identification and data
sharing and will be piloted at small scale in 2024. In the future, it may be
possible to use such an ID-wallet in a lot of different situations, for example
to share your diploma’s when applying for a job, to show your driver’s license,
or to prove that you are 18+ to buy a beer.

The NL Wallet is being developed in an open and transparent way. We offer the
following channels to allow you to contribute:

- The user interface of the app is available on [Figma][3].
- The source code is published in this [GitHub repository][4].
- More information, events and discussions can be found on [Pleio][5].

Feel free to look around and share your [feedback and ideas][6].

[[_TOC_]]

## About progress

As this project is a work in progress, you will find that the different
components are at different levels of maturity. Most notably, the user interface
is always a few steps ahead of the software under the hood. To put it simply, we
incrementally add functionality to the wallet in three steps:

1. We design the user interface of a piece of functionality in Figma. This is
   purely graphical, ideal for quick iterations.
2. We then build the user interface in the app displaying dummy data and using
   mocked logic. This makes it fast and easy to explore, demonstrate and discuss
   different scenarios and possibilities.
3. We then replace the mocked logic with actual working software, still using
   dummy data. This allows us to prove the app works and is secure.

Once the first version of the app is complete, thoroughly tested and considered
secure, we can fill it with real data and pilot it in real life scenarios.

## Current release

See the [releases page][7]for the latest release. You can follow the latest
work by subscribing to the releases of this GitHub repository at the top of this
page.

## Documentation

We have a dedicated documentation section [here][26]. In more general terms,
with the NL reference wallet we want to achieve the following things:

- We want to validate the feasibility of the framework as proposed in the EU.
- We want to explore how we can set the bar in terms of privacy protection,
  security, usability and inclusion.
- We want to learn what this development means for citizens, businesses, other
  governments and public service providers.
- We want to help citizens, especially those with special needs, in the best way
  possible.
- We want to offer a testing ground for a variety of use cases.
- We want to share the lessons we learn with the public and share them with the
  EU community.

If you want to learn more about the NL Wallet development, please read the
background information on the Pleio hub. The development of the user flows and
screens can be followed through [Figma][3].

## Licensing

The source code of the NL Wallet is released under the [EUPL license][8]. The
documentation is released under the [CC0 license](./LICENSES/CC0-1.0.txt).
Please see the [.reuse/dep5][9] file for more details, which follows the
[Reuse specfication][10].

## Contributing

We’re releasing the source code with the explicit intention of allowing
contributions. The coordination of the project lies with the development team of
the European Digital Identity Progam, but we’re open to all contributions. You
can directly create a new Pull Request via Github, or contact the community
manager via [edi@minbzk.nl][11].

The development team works on the repository in a private fork (for reasons of
compliance with existing processes) and shares its work as often as possible. If
you watch the repository on GitHub, you will be notified of a new release. We
will also send a notification through Pleio.

Although we are open to contributions, please consider the nature of this
project as outlined in this Readme. At this stage the most useful way to
contribute to the project is to participate on our community site
[edi.pleio.nl][5], and visit our [EDI Meet-ups and/or Heartbeats][12].

If you plan to make non-trivial changes, we recommend that you open an issue
beforehand where we can discuss your planned changes. This increases the chance
that we might be able to use your contribution (or it avoids doing work if there
are reasons why we wouldn't be able to use it).

Note that all commits should be signed using a GPG key.

## Getting started

This section contains the general setup requirements of the project. For more
information on how to configure specific components like [wallet app][13],
[wallet core][14], [wallet_web][15], and [wallet_provider][16], please see the
corresponding README files.

The app's UI is build using Flutter, but to avoid tying the app to Flutter &
Dart, all core business logic is build using Rust. This gives us the more
flexibility to migrate to completely native iOS/Android app's if the need
arises. This does mean building the app is slightly more complex than a simple
`flutter run`. This section describes how to set up your environment.

### System requirements

The various components of NL Wallet have different requirements. To make sure
things run correctly, you need to take the following system requirements into
account.

#### Mobile apps

Our mobile apps require at least the following operating system versions:

- Android 7.0 (API-level 24)
- iOS 14.0

The app does not put a particulary heavy load on the device, so CPU and memory
requirements are low to average. Note that this is subject to change.

#### Wallet web

The wallet_web frontend helper library effectively runs in the browser of a
person that wants to interact with a relying party that integrates with the
NL Wallet platform. As such, wallet_web has requirements on the minimum browser
version supported:

- Firefox 60.9+
- Chrome 109+
- Edge 109+
- Safari 13+

Note that the above are not recommendations, but simply a statement about the
*minimum* version we have some confidence in running correctly. We *always*
recommend that you run the latest stable browser your platform offers. Also note
that the above is subject to change.

#### Backend services

We have various backend services, mostly built in Rust that make up the wallet
platform. In general, we build binaries for `glibc` and `musl` Linux-based
distributions. We've found that usually the musl binary will work on almost
anything, but the glibc binary really requires a glibc-based distribution.

##### Operating systems we use and test our builds on

- Alpine 3.x
- Arch Linux (any current)
- Debian 12+
- RHEL 8+ (and derivatives)

##### Services we depend on

- PostgreSQL 10+ (first version with `jsonb` support)
- RDO-MAX v2.13.x+ (see [their repository][27] for details)
- BrpProxy v2.1.x (see [their repository][28] for details)

Specifically for PostgreSQL you need to consider storage requirements. Our
database-backed services are wallet_provider and wallet_server (i.e.,
verification_server and pid_issuer). They have a very simple database layout.
A good ballpark figure is to allocate 100GiB for a wallet_provider instance and
10GiB for a wallet_server instance. Of course, these requirements will change
with time and duration of usage, and are subject to change.

##### Static configuration service

- configuration_server (simple static webserver, no particular requirements)

##### Rust-based backend services

- wallet_provider
- wallet_server (verification_server, pid_issuer)
- mock_relying_party
- gba_hc_converter

The above Rust-based services require a regular Linux machine or container
based on one of the aforementioned operating systems. Memory requirements of
these services are very low (we're seeing 20 to 50 megabytes of usage on our
Kubernetes clusters, but of course it depends on usage too). The storage
requirements are effectively non-existent due to usage of PostgreSQL for state.

#### Network connectivity

For end-users, an internet connection is required to use the disclosure and
issueance features of the wallet app. For relying parties and issuers, who want
to obtain disclosed attributes and issue attributes respectively, the same
requirement holds.

### Development requirements

To do development work on the NL reference wallet, you need to following tools:

- Flutter
- Rust (incl. additional targets)
- Android SDK + NDK (for Android builds)
- Xcode (for iOS builds)
- Docker (to run supporting services)

In the coming sub-sections we will document how to install and configure those.

#### Flutter

To install Flutter follow this [installation guide][17]. You can validate your
initial setup by running `flutter doctor`.

**Manage your local Flutter version using Flutter Version Manager (FVM)** FVM
is a simple CLI to manage Flutter SDK versions per project. It enables fast
switching between Flutter versions and pin them to your Flutter project. When
using FVM; all Flutter related commands need to be prefixed with `fvm`, e.g.
`fvm flutter run`.

_Optional step:_ To install FVM follow this [installation guide][18]. You can
validate your initial setup by running `fvm flutter doctor` after installation.
Select `[Y]es` when asked to install the pinned Flutter version defined in
[fvm_config.json][19].

Note that FVM only pins the Flutter version for local development, not for the
CI pipelines.

#### Rust

To install Rust and Cargo using `rustup`, follow the [installation guide][20].
After installation, make sure to add the following targets:

- For iOS: `rustup target add aarch64-apple-ios x86_64-apple-ios`
- For iOS simulator: `rustup target add aarch64-apple-ios-sim`
- For Android:
  `rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`

#### Android

To build for Android you need to have the Android SDK and NDK installed on your
system. Likely the easiest way to do so is:

1. Install [Android Studio][21] (includes SDK)
2. Open Android Studio,
    1. Tools -> SDK Manager,
    2. Select 'SDK Tools' Tab,
    3. Check latest 'NDK (Side by side)' in the list (>= v25.2.9519653),
    4. Hit 'apply' to install.
3. Add `export ANDROID_HOME="$HOME/Library/Android/sdk"` to your
   `~/.bash_profile` or `~/.zshrc`, this will make sure the Android SDK is
   available in your path and automatically picks up the NDK version you
   installed in the previous step
4. Tell `gradle` where it can find the NDK by updating your `gradle.properties`
   e.g.
   `echo "NDK_HOME=\"$HOME/Library/Android/sdk/ndk/{ndk_version}\"" >> ~/.gradle/gradle.properties`
5. Run `cargo install cargo-ndk` (>= v3.0.0) to be able to build the Rust code
   for Android
6. Optional: let Android Studio use Flutter SDK installed via FVM by following
   [these steps][22]
7. Run `$HOME/Library/Android/sdk/emulator/emulator -list-avds` to list the
   installed devices
8. Start emulator: `${HOME}/Library/Android/sdk/emulator/emulator <some-avd>`

#### Xcode

1. Install [Xcode][23]
2. Follow the steps to install iOS simulators
3. Start the simulator using `open -a Simulator`

#### Docker

Make sure you can run Docker on your development system. You can follow the
[getting started guide][25] to do so.

### Supporting services

The Wallet app needs several supporting services to run, and also requires the
user to log in using DigiD in order to create the Wallet. The services are the
following:

- wallet_provider
- verification_server
- pid_issuer
- mock_relying_party
- digid-connector
- configuration_server
- brpproxy
- gba_hc_converter
- rdo-max
- postgresql

All these applications will need to be configured correctly. A local development
environment with automatic configuration of all the above services can be set up
using the `scripts/setup-devenv.sh` and `scripts/start-devenv.sh` scripts.

#### Configuring a local development environment

The `setup-devenv.sh` script will configure the digid-connector to listen on
https://localhost:8006/ . Note the https in the URL, which is provided using
self-signed certificates.

Besides that, the development setup runs without using TLS. Therefore, the
feature `allow_insecure_url` enables the possibility to use a return URL with
the scheme `http` (while normally `https` is only allowed).

The local wallet can be connected to Sentry for crash and error reporting by
setting the `SENTRY_DSN` environment variable.

Additionally, the `wallet` crate offers the `config_env` feature to aid during
local development, which does the following:

* Any constant defined in the file `data.rs` can be overridden by an
  environment variable of the same name at compile time.
* Additional environment variables are read from a file named `.env`
  in the `wallet` crate directory, if present.

#### Starting a local development environment

The individual services of the development environment can be started using
`start-devenv.sh`. Tip: use the `--help` commandline option to show the help
output.

#### Starting an Android Emulator instance

In order to connect to our locally running services from the Android Emulator on
the domain `localhost`, some port mappings have to be made. Note that this must
be done every time the Android Emulator is restarted.

This is automated in the script: `scrips/map_android_ports.sh`.

The `setup-devenv.sh` script will automatically run this script when it detects
the `adb` command.

However, when the Android Emulator has been restarted, one can just run
`map_android_ports.sh`.

#### Validate local development environment

After doing the above `flutter doctor` should report that at least the following
are installed successfully:

- Flutter
- Android toolchain
- Xcode

You should now be able to launch an Android Emulator or iOS Simulator and run
the app by following these steps:

- `cd wallet_app`
- `flutter pub get`
- `flutter run`! 🎉

#### Running GitLab CI locally (optional)

In order to run and validate jobs from the GitLab CI locally on a development
machine, the `gitlab-ci-local` tool may be used. Follow the installation
instructions for it [here][24].

The environment variables that are necessary to run the CI jobs need to be
specified by copying and populating the example YAML file:

```sh
cp .gitlab-ci-local-variables.example.yml .gitlab-ci-local-variables.yml
```

Make sure that Docker is running and configure it so that containers have a
maximum memory size of at least 16GB. Log into Harbor, where the docker images
are hosted:

```sh
docker login -u <HARBOR USER> -p <HARBOR CLI SECRET> <HARBOR HOSTNAME>
```

Now, any job from GitLab CI can be run localy, e.g.:

```sh
gitlab-ci-local test-rust
```

## File structure

### Code files

All `Dart` code goes in the `wallet_app/lib/` directory and their appropriate
sub-directories.

All `Rust` code goes in the `wallet_core/` directory and their appropriate
sub-directories.

### Flutter <-> Rust Bridge

Communication between the Flutter and Rust layers relies on the
`flutter_rust_bridge` package, the bridge code is generated. The definition of
this bridge can is located at `/wallet_core/src/api.rs` and generation is done
with the following command:

```
cargo run --manifest-path wallet_core/flutter_rust_bridge_codegen/Cargo.toml
```

The generated code is currently checked in, so that generation only has to be
performed when the API changes.

## Conventions

### Git

#### Commit message

- Capitalize the subject line
- Use the imperative mood in the subject line
- Do not end the subject line with a period
- Wrap lines at 72 characters

#### Branch names

- Prefix the branch name with the Jira code for the story or subtask the branch
  relates to. If there is no story or subtask, strongly consider making one or
  forego the prefix.
- The rest of the branch name should be a short description of the purpose of
  the branch, in lowercase and separated by hyphens. The description should be
  clear enough that any reader should understand it without having to look up
  the Jira ticket. Consider starting the description with the component that is
  being worked on, e.g. `ci-` or `core-`.

Example of a branch name: **PVW-123-wp-teapot-status-code**

#### PR title

See [commit message](#commit-message).

#### PR merge

- Default to squash merge (combined with PR title conventions)

## Distribution

Follow these steps to (force) distribute internal `alpha` & `beta` builds that
target the Android platform":

### Alpha

> Use `Alpha` distribution at any time during development cycle.

* Push commit of your choosing to: alpha/v{X.Y.Z}
* After the GitHub Action has completed successfully; install the release via
  F-Droid repo

### Beta

> Use `Beta` distribution at the end of a sprint cycle; to represent the sprint
> demo version.

* `$ git fetch && git pull`
* Push `main` branch to: beta/v{X.Y.Z}
* After the GitHub Action has completed successfully; install the release via
  F-Droid repo

## Troubleshooting

### Initial checkout / branch switch

Generate/update localisation files (to compile/run the project successfully):

    $ flutter gen-l10n

## References

Below you'll find a collection of links which we reference to through the entire
text. Note that they don't display when rendered within a website, you need to
read the text in a regular text editor or pager to see them.

[1]: https://www.digitaleoverheid.nl/kabinetsbeleid-digitalisering/werkagenda/

[2]: https://www.rijksoverheid.nl/onderwerpen/inloggen-europese-economische-ruimte-eer-eidas/alles-wat-u-moet-weten-over-eidas/

[3]: https://www.figma.com/design/4efsEQFFJqenB80OKqfmdl/20241126_Release_UI_NLWallet?node-id=1-3716&t=HhqTBGpCrSVcW8ku-1

[4]: https://github.com/MinBZK/nl-wallet/

[5]: https://edi.pleio.nl/

[6]: mailto:edi@minbzk.nl?subject=Feedback%20or%20ideas

[7]: releases

[8]: ./LICENSES/EUPL-1.2.txt

[9]: ./.reuse/dep5

[10]: https://reuse.software/spec/

[11]: mailto:edi@minbzk.nl?subject=Contribution%20via%20GitHub

[12]: https://edi.pleio.nl/events/

[13]: ./wallet_app/README.md

[14]: ./wallet_core/README.md

[15]: ./wallet_web/README.md

[16]: ./wallet_core/wallet_provider/README.md

[17]: https://flutter.dev/docs/get-started/install

[18]: https://fvm.app/documentation/getting-started/installation

[19]: ./wallet_app/.fvm/fvm_config.json

[20]: https://www.rust-lang.org/tools/install

[21]: https://developer.android.com/studio

[22]: https://fvm.app/documentation/getting-started/configuration

[23]: https://apps.apple.com/us/app/xcode/id497799835?mt=12

[24]: https://github.com/firecow/gitlab-ci-local#installation

[25]: https://docs.docker.com/get-started/get-docker/

[26]: ./documentation/index.md

[27]: https://github.com/minvws/nl-rdo-max

[28]: https://github.com/BRP-API/Haal-Centraal-BRP-bevragen
