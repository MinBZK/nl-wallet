HW Keystore
===========

This crate allows native Android and iOS functionality to be called from the Rust core.
Currently the functionality is the following:

* Hardware backed ECDSA private keys can be created
* The derived public keys for these private keys can be retrieved
* Arbitrary payloads can be signed with the private key

This functionality is provided by traits that have multiple concrete implementations.

# Features

The crate contains the following features:

* `hardware` (enabled by default): This compiles the hardware backed implementations, which uses `uniffi` to bridge to either Android or iOS native code.
* `software`: This compiles a software fallback implementation, which can be used during both testing and local development.
* `integration-test`: This should only be enabled when running integration tests (see below).

# Components

The functionality is split into multiple parts that are compiled in distinct steps and ultimately combined by the linker when building the app.
As there are slight differences between Android and iOS, they are described separately below.

## Android

TBD

## iOS

First, there is the Swift implementation, which is contained within a small Xcode project that produces a static library (i.e. a `.a` file).
This project and static library are called `HWKeyStore`.
When compiling this Xcode project, Swift code will automatically be generated from the UDL file included in the crate through `uniffi-bindgen`.
In Swift, a singleton class wraps the `init_hw_keystore()` that needs to be called on app startup, which lets Rust know how to call the native code.

Then there is the Rust code that accepts the `init_hw_keystore()` function call and allows a consumer of this crate to call to native code.
This also uses `uniffi` during compilation to generate the necessary Rust code from the UDL file.
The `hw_keystore` crate is included directly in compilation of the `rust_core` crate, which in turn produces another static library.

The two are combined in the main Xcore project of the app.
The smaller Xcode project mentioned above is included as a dependency of this project, while the `rust_core` crate is compiled as a build step within this project.
The main project creates an instance of the singleton class on app startup in its `AppDelegate`.
Finally both static libraries that are produced are linked together with the main app binary, causing all of the required symbols to be resolved.

The final process can be visualised as follows:

```
Wallet Xcode Project --> rust_core --> hw_keystore
            |                               ^
            |                               | (uniffi)
            \----->     HWKeyStore     -----/
```

# Integration test

## Software fallback

The crate contains an integration test for the software fallback, which can be run using `cargo test --features software,integration-test`.
This test simply uses the crate to create a new private key, get its public key, sign a payload and then verify the returned signature using the public key.
Note that the `integration-test` feature is necessary so that some helper code is included in the build.

## Android

TBD

## iOS

In order to run the same integration test either in the iOS simulator or on actual hardware, a test target is included in the `HWKeyStore` Xcode project.
This test target compiles the `hw_keystore` crate directly and includes it in a test binary (a step that is normally done by the main app Xcode project).
When run, the test target calls out to Rust code to start running the integration test, which in turn calls the Swift implementation.

This can be visualised as follows:

```
HWKeyStore test --> hw_keystore
      |                   ^
      |                   | (uniffi)
      \-->  HWKeyStore  --/
```
