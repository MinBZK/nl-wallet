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
- Project documentation is available on [Github pages][25].

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

See the [releases page][7] for the latest release. You can follow the latest
work by subscribing to the releases of this GitHub repository at the top of this
page.

## Documentation

We have a dedicated [documentation site][25]. In more general terms, with the
NL reference wallet we want to achieve the following things:

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
- RDO-MAX v2.13.x+ (see [their repository][26] for details)
- BrpProxy v2.1.x (see [their repository][27] for details)

Specifically for PostgreSQL you need to consider storage requirements. Our
database-backed services are `wallet_provider`, `verification_server`,
`issuance_server`, `demo_issuer` (usually not built for production environments)
and `pid_issuer`. They have a very simple database layout. A good ballpark
figure is to allocate 100GiB for a wallet_provider instance and 10GiB for
instances of the verification_server, issuance_server or pid_issuer. Of course,
these requirements will change with time and duration of usage, and are subject
to change. Also note that these size requirements assume somewhat serious usage
- for development purposes you can make do with a lot less.

##### Rust-based backend services

- wallet_provider
- verification_server
- issuance_server
- pid_issuer
- demo_relying_party
- demo_issuer
- gba_hc_converter

The above Rust-based services require a regular Linux machine or container
based on one of the aforementioned operating systems. Memory requirements of
these services are very low (we're seeing 20 to 50 megabytes of usage on our
Kubernetes clusters, but of course it depends on usage too). The storage
requirements are effectively non-existent due to usage of PostgreSQL for state.

##### Additional supporting services (in addition to rust-based backend services)

The Wallet app needs several supporting services to run, and also requires the
user to log in using DigiD in order to create the Wallet. The services are the
following:

- digid-connector (rdo-max)
- configuration_server
- gba_hc_converter
- brpproxy
- postgresql

All these applications will need to be configured correctly. A local development
environment with automatic configuration of all the above services can be set up
using the `scripts/setup-devenv.sh` and `scripts/start-devenv.sh` scripts, which
we'll document later in this Readme.

#### Network connectivity

For end-users, an internet connection is required to use the disclosure and
issuance features of the wallet app. For relying parties and issuers, who want
to obtain disclosed attributes and issue attributes respectively, the same
requirement holds.

### Development requirements

To do development work on the NL reference wallet, you need to following tools:

- Rust (including additional targets and utilities)
- Android SDK + NDK (for Android builds)
- Xcode (for iOS builds)
- Flutter
- Docker (to run supporting services)
- PKCS #11 library

In the following sub-sections we will document how to install and configure
these.

#### Rust

To install Rust and Cargo using `rustup`, follow the [installation guide][20].
After installation, make sure to add the following targets:

- For iOS: `rustup target add aarch64-apple-ios x86_64-apple-ios`
- For iOS simulator: `rustup target add aarch64-apple-ios-sim`
- For Android: `rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`

Make sure `rustc`, and `cargo` are on your path and run the following commands
to install a few additional utilities we use:

```shell
cargo install --locked cargo-edit cargo-expand cargo-ndk cargo-nextest sea-orm-cli
cargo install --locked --version 2.11.1 flutter_rust_bridge_codegen
cargo install --list
```

#### Android

To build for Android you need to have the Android SDK and NDK installed on your
system. You can [download Android Studio here][21]. Note the install location
and set the `ANDROID_HOME` environment variable to point to that installation
location. The following is an example that assumes you installed Android Studio
in `/opt/android`, sets the necessary variables, and adds certain
Android-specific tools and utilities to your path:

```shell
export ANDROID_HOME="/opt/android"
export ANDROID_NDK_HOME="$(find "$ANDROID_HOME/ndk" -maxdepth 1 -type d | sort -V | tail -n1)"
export PATH="$PATH:$ANDROID_HOME/platform-tools:$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_NDK_HOME"
```

After having done the above, you will have tools like `adb`, `sdkmanager`, and
`ndk-build` on your path.

With `sdkmanager --list_installed` you can list the installed Android SDK
components. Now let's install some extra components we'll need:

```shell
# Install all needed packages. On intel/amd, x86_64 is
# fine, on arm64, use arm64-v8a for the system-image.
sdkmanager --install \
'build-tools;36.0.0' \
'cmdline-tools;latest' \
'emulator' \
'ndk;28.2.13676358' \
'platform-tools' \
'platforms;android-36' \
'sources;android-36' \
"system-images;android-36;google_apis_playstore;$(uname -m|sed s/arm64/arm64-v8a/)"
```

Note: The `ndk` version was the latest non-release-candidate version as of this
writing (2025-08-28); If there is a newer stable version, feel free to use that.
Same for the android version - it's 36 as of this writing, but newer might be
available when you read this - feel free to use that too.

You will need to create an Android virtual device (AVD) so you can run the
emulator. To do that, do the following:

  1. Start Android Studio, open or create any project;
  2. Click "File" pulldown menu, then "Tool Windows" -> "Device Manager";
  3. Click on "+", then "Create Virtual Device", select "Pixel 3"
  4. For "Name", choose something descriptive like "pixel3-x86_64-android16-api36"
  5. For "API", select latest available, like "API 36.0"
  6. For "Services", select "Google Play Store"
  7. For "System Image", choose whichever latest Google Play enabled image is
     available. For example: "Google Play <arch> System Image", "API 36.0"
  8. Click on the "Additional Settings" tab
  9. Set "Expanded Storage" to "Custom", "12", "GB"
 10. Set "RAM" to "4", "GB"
 11. Set "VM heap size" to "512", "MB"
 12. Click on the blue "Finish" button

When you run `$ANDROID_HOME/emulator/emulator -list-avds` on the command-line,
you should see your just-created virtual device show up.

#### Xcode

1. Install [Xcode][23]
2. Follow the steps to install iOS simulators
3. Start the simulator using `open -a Simulator`

#### Flutter

To install Flutter follow this [installation guide][17]. You can validate your
initial setup by making sure `flutter` is on your path, and then running
`flutter doctor` which will tell you if all is well with its dependencies. Make
sure `flutter doctor` has no complaints or warnings; specifically, it needs to
find the Android and/or Xcode related components (in the case of Android, it
needs to find the SDK toolchain and Android Studio itself).

##### Manage your local Flutter version using FVM (optional)

FVM is a simple CLI to manage Flutter SDK versions per project. It enables fast
switching between Flutter versions and pin them to your Flutter project. When
using FVM; all Flutter related commands need to be prefixed with `fvm`, e.g.
`fvm flutter run`.

To install FVM follow this [installation guide][18]. You can validate your
initial setup by running `fvm flutter doctor` after installation. Select `[Y]es`
when asked to install the pinned Flutter version defined in
[fvm_config.json][19].

Note that FVM only pins the Flutter version for local development, not for the
CI pipelines.

#### Docker

Make sure you can run Docker on your development system. You can follow the
[getting started guide][24] to do so. Make sure `docker` is on your path.

#### PKCS #11 library

The wallet_provider is designed to use a HSM with the [PKCS #11][29] API. For
local development we use the [SoftHSMv2][28] library. As of this writing
(2025-08-28), the latest actually released version of this library is more than
five years old and does *not* work in combination with the latest openssl.
Therefore, you need to compile the library from source, specifically, from the
`develop` branch, and set the correct `HSM_LIBRARY_PATH` (if the setup script
does not detect the library).

#### Starting an Android emulator instance

Assuming you've followed the steps in the [setting up Android Studio](#android)
section, you can now run the emulator. You can do that from within Android
Studio, or, as shown below, from the command-line:

```shell
# Optional, but required on Linux (QT_QPA_PLATFORM=wayland segfaults emulator).
export QT_QPA_PLATFORM=xcb

# List configured Android virtual devices you can run.
# You can create these in Android Studio -> Device Manager.
"$ANDROID_HOME/emulator/emulator" -list-avds

# Start an avd named pixel3-x86_64-android16-api36 (can be named anything,
# should have appeared in previous listing). We use the -wipe-data flag to
# start with a clean data partition. Issues have been observed with default
# snapshotting behavior, which is why we start with no snap storage and no
# automatic snapshotting. Audio is also not strictly needed. The -no-window
# flag is optional and good enough for running connected android tests. If
# you do want to see the phone gui, you need to remove the -no-window option.
"$ANDROID_HOME/emulator/emulator" -avd pixel3-x86_64-android16-api36 -wipe-data -no-snapstorage -no-snapshot -no-audio -no-window


# Optional, disables bluetooth on emulator to avoid observed interference
# with bluetooth on host.
adb shell cmd bluetooth_manager disable
```

In order to connect to our locally running services from within the running
Android emulator, some port mappings have to be made (note that this must
be done every time the Android emulator is restarted). This is automated in
our `map_android_ports.sh` script, which our `setup-devenv.sh` script will
call automatically when it detects `adb` on the path.

#### Using your own PostgreSQL service (optional)

The `start-devenv.sh` script can set up a dockerized PostgreSQL database for you
or you could opt to configure it yourself. If you want to set it up yourself,
make sure to have it running on localhost, port 5432. Here's an example `docker`
command to bring up a latest version PostgreSQL container:

```shell
docker run --name postgres --volume postgres:/var/lib/postgresql/data \
--rm --publish 5432:5432 --env POSTGRES_PASSWORD=verysecret postgres
```

The above docker command will start a Docker container running the latest
postgres image, bound to localhost on port 5432, with `verysecret` as a
password for the `postgres` system user. There are no other users or
databases yet, just the defaults.


##### Create a dedicated wallet database user (optional)

If you want, you can create a dedicated `wallet` user for the various databases
we need to run, instead of the `postgres` system user:

```shell
export PGPASSWORD=verysecret
psql -h localhost -U postgres -c "create user wallet with password 'verysecret';"
psql -h localhost -U postgres -c "create database pid_issuer owner wallet;"
psql -h localhost -U postgres -c "create database issuance_server owner wallet;"
psql -h localhost -U postgres -c "create database verification_server owner wallet;"
psql -h localhost -U postgres -c "create database wallet_provider owner wallet;"
psql -h localhost -U postgres -d wallet_provider -c "create extension \"uuid-ossp\" schema public;"
```

We need to set `DB_USERNAME` and `DB_PASSWORD`, which are used by the
`setup-devenv.sh` script to initialize the schemas of the above databases:

```shell
export DB_USERNAME=wallet DB_PASSWORD=verysecret
```

#### Configuring a local development environment

After having done all of the above (i.e., you have Rust and Flutter installed,
you have Docker up and running, configured a PostgreSQL database and installed
Android Studio and/or Xcode and you're running an Android Emulator or the iOS
simulator), you are almost ready to configure the local development environment
with  the help of our `setup-devenv.sh` script. There are two more optional
environment variables to consider setting:

```shell
# Where the rdo-max git repository is cloned to.
export DIGID_CONNECTOR_PATH="$HOME/Desktop/rdo-max"

# For android, by default, we build x86_64, arm64-v8a and armeabi-v7a,
# This is quite time consuming. You can opt to only build for one of
# these. The below would usually choose either x86_64 or arm64-v8a.
export ANDROID_NDK_TARGETS="$(uname -m|sed s/arm64/arm64-v8a/)"
```

To run the setup script, enter the git repository directory and go:

```shell
cd nl-wallet
scripts/setup-devenv.sh
```

##### Additional notes about local development

The `setup-devenv.sh` script will configure the digid-connector to listen on
https://localhost:8006/ . Note the https in the URL, which is provided using
self-signed certificates.

Besides that, the development setup runs without using TLS. Therefore, the
feature `allow_insecure_url` enables the possibility to use a return URL with
the scheme `http` (while normally only `https` is allowed).

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

For example, after having run `setup-devenv.sh`, you could run `start-devenv.sh`
with the `--all`, which starts everything, including a dockerized PostgreSQL
database and the NL-Wallet flutter app itself (which means that you need to have
an Android emulator or iOS simulator running already, or you have otherwise
connected a mobile target for flutter to connect and deploy to).

Conveniently, you could run with the `--default` flag, which starts everything
except PostgreSQL and the NL-Wallet app. We use this mode of running a lot when
developing locally: simply start your own PostgreSQL, dockerized or locally and
then `scripts/start-devenv.sh --default` to bring up all backend services. You
can then work on the code and simply `flutter run` in the `wallet_app` directory
once you're ready to see the mobile app running. So:

```shell
cd nl-wallet
scripts/start-devenv.sh --default
```

#### Running Rust tests

To run both our unit- and integration tests, we can run the following (note: we
use `cargo nextest` here, but you can use regular `cargo test` too):

```shell
cd nl-wallet
cargo nextest run --manifest-path wallet_core/Cargo.toml --features integration_test
```

Note that the above runs both unit- and integration tests. The latter requires
[a running backend](#starting-a-local-development-environment). If you only
want to run the unit tests, simply don't specify `--features integration_test`.

#### Running connected Android tests

Make sure your Android emulator is up and running, and then run the following:

```shell
cd nl-wallet/wallet_core/wallet/platform_support/android
./gradlew testDebugUnitTest connectedDebugAndroidTest
```

You should see gradle build running. This will create a special kind of APK that
is uploaded to the connecte (emulated) Android device, which the tests interact
with.

#### Run the app on emulator or simulator

Make sure you have an Android Emulator or iOS Simulator running and start the
app:

```shell
cd nl-wallet/wallet_app
flutter pub get
flutter run # 🎉
```

After a few moments, you should see the NL-Wallet app appear on your (emulated
or simulated) device. The app will interact with the backend services started
by `start-devenv.sh`.

#### Generate Flutter-Rust bridge code (optional)

Communication between the Flutter and Rust layers relies on the
`flutter_rust_bridge` package. The bridge code is generated. The definition of
this bridge is located at `/wallet_core/flutter_api/src/api` and generation is
done with the following command:

```shell
# Enter the nl-wallet git directory
cd nl-wallet

# Generate bridge code (note: take a look at git diff afterwards. If flutter
# analyze later in this guide fails, revert with git checkout -- wallet_app).
flutter_rust_bridge_codegen generate --config-file wallet_app/flutter_rust_bridge.yaml
```

**A note for Linux users, specifically on non-Debian systems:** You need to set
`CPATH` to workaround issues when using `flutter_rust_bridge_codegen`. Note that
these issues have been observed under both `flutter_rust_bridge_codegen` 1.x and
2.x. For more background on this, see [here][30], and [here][31]. To set `CPATH`
for `flutter_rust_bridge_codegen`, you can run as follows:

```shell
CPATH="$(clang -v 2>&1 | grep "Selected GCC installation" | rev | cut -d' ' -f1 | rev)/include" \
flutter_rust_bridge_codegen generate --config-file wallet_app/flutter_rust_bridge.yaml
```

Note that the generated code is checked into our git repository, so generation
only has to be performed when the API code changes.

#### Format Rust and Dart source code files (optional)

You can format Rust code as follows:

```shell
cd nl-wallet
cargo clippy --manifest-path wallet_core/Cargo.toml --all-features --tests -- -Dwarnings
find wallet_core -mindepth 2 -type f -name Cargo.toml -print0 | xargs -0 -n1 cargo fmt --manifest-path
```

You can format Dart code as follows (note that formatting output is different
when `flutter pub get` never ran before):

```shell
cd nl-wallet/wallet_app
flutter pub get
dart format . --line-length 120
```

## File structure

### Code files

All Rust code goes in the `wallet_core/` directory and their appropriate
sub-directories.

All Dart/Flutter code goes in the `wallet_app/lib/` directory and their
appropriate sub-directories.

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

## App build configuration

The app build includes the configuration for the connection to the configuration-server (`config-server-config.json`).
Additionally it includes an initial configuration from the configuration-server (`wallet-server.json`).

Next to these configuration files the build can be configured with:

| Name                          | Type        | Components                          | Description |
|-------------------------------|-------------|-------------------------------------|-------------|
| APPLE_ATTESTATION_ENVIRONMENT | Env         | Apple Entitlement, Cargo option_env | Attestation environment for iOS (development / production). Only used in Cargo if `fake_attestation` is set. Default is `development`, which is ignored by Testflight and App Store. See [wallet_app/README.md](wallet_app/README.md#ios-1) for more info. |
| UL_HOSTNAME                   | Env         | Apple Entitlement                   | Universal Link hostname (iOS only). |
| universal_link_base           | Option      | Android                             | Universal Link hostname (Android only), passed via Dart define as UL_HOSTNAME. |
| UNIVERSAL_LINK_BASE           | Env         | Cargo option_env                    | Universal Link base URL used in Wallet Core. |
| ALLOW_INSECURE_URL            | _Dart only_ | Cargo feature                       | Whether to allow http urls in Wallet Core (passed via Dart define as ALLOW_INSECURE_URL via Xcode / build.gradle as `wallet/allow_insecure_url`). Defaults to `false`. |
| CONFIG_ENV                    | Env         | Cargo build                         | The configuration environment name (should match the environment in `config-server-config.json` and `wallet-config.json`). Defaults to `dev`. |
| SENTRY_AUTH_TOKEN             | Env         | Flutter                             | [Sentry Auth Token](https://docs.sentry.io/account/auth-tokens/), empty if not enabled (read by Dart plugin via environment). |
| SENTRY_PROJECT                | Env         | Flutter                             | [Sentry Project](https://docs.sentry.io/concepts/key-terms/key-terms/), empty if not enabled (read by Dart plugin via environment). |
| SENTRY_ORG                    | Env         | Flutter                             | Sentry Organization slug, empty if not enabled (read by Dart plugin via environment). |
| SENTRY_URL                    | Env         | Flutter                             | Sentry URL, empty if not enabled (read by Dart plugin via environment). |
| SENTRY_DSN                    | Env         | Flutter                             | [Sentry Data Source Name](https://docs.sentry.io/concepts/key-terms/dsn-explainer/), empty if not enabled (passed via Dart define as SENTRY_DSN). |
| SENTRY_ENVIRONMENT            | Env         | Flutter                             | [Sentry Environment](https://docs.sentry.io/concepts/key-terms/environments/) (passed via Dart define as SENTRY_ENVIRONMENT). |
| SENTRY_RELEASE                | Env         | Flutter                             | [Sentry Release](https://docs.sentry.io/product/releases/) (passed via Dart define as SENTRY_RELEASE). |
| build                         | Option      | iOS / Android build                 | The build number of the build (should be strictly increasing when submitting to App or Play Store). Defaults to `0`. |
| version                       | Option      | iOS / Android build                 | The version of the build (should be semver). Defaults to the version in the [pubspec.yaml](wallet_app/pubspec.yaml). |
| app_name                      | Option      | iOS / Android build                 | The app name (passed via environment as APP_NAME). Defaults to the `NL Wallet`. |
| application_id                | Option      | Android build                       | The Android application id (passed via environment as APPLICATION_ID). Defaults to `nl.ictu.edi.wallet.latest`. |
| bundle_id                     | Option      | iOS                                 | The iOS bundle id (changed via update_code_signing_settings lane). Defaults to `nl.ictu.edi.wallet.latest`. |
| build_mode                    | Option      | Flutter                             | The build mode of Flutter (debug / profile / release). Defaults to `release`. |
| file_format                   | Option      | Android build                       | File format (aab / apk)  for Android build. Defaults to `aab`. |
| fake_attestation              | Option      | Cargo feature                       | Whether to use a fake Apple attestation (passed via Dart define as FAKE_ATTESTATION, via Xcode as `wallet/fake_attestation`). Defaults to `true` if built for Simulator otherwise `false`. |
| mock                          | Option      | Flutter                             | Whether or not to use mock mode in Flutter (passed via Dart define as MOCK_REPOSITORIES). Defaults to `false`. |
| demo_index_url                | Option      | Flutter                             | The URL to launch the demo index page in Browser for tests (passed via Dart define as DEMO_INDEX_URL). |


## Troubleshooting

### Initial checkout / branch switch

Generate/update localisation files (to compile/run the project successfully):

    $ flutter gen-l10n

## References

Below you'll find a collection of links which we reference to through the entire
text. Note that they don't display when rendered within a website, you need to
read the text in a regular text editor or pager to see them.

[1]: https://www.digitaleoverheid.nl/kabinetsbeleid-digitalisering/werkagenda/

[2]: https://www.rijksoverheid.nl/onderwerpen/inloggen-europese-economische-ruimte/alles-wat-u-moet-weten-over-eidas

[3]: https://www.figma.com/design/Pv7yVW8jg26dgW1IWjVgt1/20250625_Release_UI_NLWallet?node-id=1-3716&t=eVkXvIquG6fOOLOF-1

[4]: https://github.com/MinBZK/nl-wallet/

[5]: https://edi.pleio.nl/

[6]: mailto:edi@minbzk.nl?subject=Feedback%20or%20ideas

[7]: /../../releases

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

[24]: https://docs.docker.com/get-started/get-docker/

[25]: https://minbzk.github.io/nl-wallet/

[26]: https://github.com/minvws/nl-rdo-max

[27]: https://github.com/BRP-API/Haal-Centraal-BRP-bevragen

[28]: https://github.com/softhsm/SoftHSMv2

[29]: https://en.wikipedia.org/wiki/PKCS_11

[30]: https://github.com/fzyzcjy/flutter_rust_bridge/issues/1375

[31]: https://cjycode.com/flutter_rust_bridge/v1/integrate/deps.html#non-debian-linux
