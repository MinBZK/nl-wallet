NL-Wallet Platform Support
==========================

This crate allows native Android and iOS functionality to be called from the wallet core.

# Rustdoc

Documentation on the types in this crate can be generated and inspected using the following command:

```bash
cargo doc --open
```

# Components

The functionality is split into multiple parts that are compiled in distinct steps and ultimately combined by the linker when building the app.
As there are slight differences between Android and iOS, they are described separately below.

## Android

The Kotlin implementation is contained within a small Android project `PlatformSupport`. This project contains a single module `platform_support` which produces:
* A shared library (i.e. a `.so` file) that contains the native code.
* Kotlin (binding) code, generated from the UDL files included in the crate through `uniffi-bindgen`.

Singleton classes wrap the initializers that need to be called on app startup (e.g. `initHwKeystore()`), which lets Rust know how to call the native code.
When compiling the main Android project of the app, the `platform_support` module is included as a dependency.

The final process can be visualised as follows:

```
Wallet Android Project --> wallet_core --> platform_support
            |                                 ^
            |                                 | (uniffi)
            \----->  PlatformSupport  -------/
```

## iOS

First, there is the Swift implementation, which is contained within a small Xcode project that produces a static library (i.e. a `.a` file).
This project and static library are called `PlatformSupport`.
When compiling this Xcode project, Swift code will automatically be generated from the UDL files included in the crate through `uniffi-bindgen`.
In Swift, singleton classes wrap the initializers that need to be called on app startup (e.g. `init_hw_keystore()`), which lets Rust know how to call the native code.

Then there is the Rust code that accepts the initializer function calls and allows a consumer of this crate to call to native code.
This also uses `uniffi` during compilation to generate the necessary Rust code from the UDL files.
The `platform_support` crate is included directly in compilation of the `wallet_core` crate, which in turn produces another static library.

The two are combined in the main Xcore project of the app.
The smaller Xcode project mentioned above is included as a dependency of this project, while the `wallet_core` crate is compiled as a build step within this project.
The main project creates instances of the singleton classes on app startup in its `AppDelegate`.
Finally both static libraries that are produced are linked together with the main app binary, causing all of the required symbols to be resolved.

The final process can be visualised as follows:

```
Wallet Xcode Project --> wallet_core --> platform_support
            |                                  ^
            |                                  | (uniffi)
            \----->  PlatformSupport  --------/
```

# Hardware Keystore

Currently the functionality of this module is the following:

* Hardware backed ECDSA private keys can be created
* The derived public keys for these private keys can be retrieved
* Arbitrary payloads can be signed with the private key

This functionality is provided by traits that have multiple concrete implementations.

## Features

The module contains the following features:

* `software`: This compiles a software fallback implementation, which can be used during both testing and local development.
* `integration_test`: This feature is automatically enabled when running integration tests, it contains some helper code.
* `hardware_integration_test`: This should only be enabled when running integration tests from either Android or iOS (see below).

## Integration tests

### Software fallback

The crate contains an integration test for the software fallback, which can be run using `cargo test --features software-integration_test`.
This test simply uses the crate to create a new private key, get its public key, sign a payload and then verify the returned signature using the public key.

### Android

To run the Android integration tests: execute `./gradlew connectedAndroidTest` from the `/wallet_core/platform_support/android` directory.

Note: if you get an error like the following:

```
AndroidTestApkInstallerPlugin: ErrorName: INSTALL_FAILED_INSUFFICIENT_STORAGE
```

You will usually find that there is enough storage on the (emulated) device. You can start an `adb` shell and clear the temporary storage:

```
adb shell "rm -rf /data/local/tmp/*"
```

Then restart `./gradlew connectedAndroidTest` and you'll find the installation succeed and the tests continue normally.

### iOS

In order to run the same integration test either in the iOS simulator or on actual hardware, a test target is included in the `PlatformSupport` Xcode project.
This test target compiles the `platform_support` crate directly and includes it in a test binary (a step that is normally done by the main app Xcode project).
When run, the test target calls out to Rust code to start running the integration test, which in turn calls the Swift implementation.

This can be visualised as follows:

```
Integration test --> platform_support
      |                      ^
      |                      | (uniffi)
      \--> PlatformSupport --/
```

Note that the attested key integration tests for iOS can only be run on a real device.
