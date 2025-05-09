# Wallet core

This contains the "shared core" part of the app.

## Project structure

The functionality of the Wallet and supporting services is spread accross multiple directories in the workspace.
The list below is not exhaustive, but is meant as a starting point for understanding the project structure.

- `configuration_server`: Wallet configuration server for local development.
- `flutter_api`: Contains `flutter_rust_bridge` bindings. This allows Flutter to use the functionality from the `wallet` crate.
- `gba_hc_converter`: Web server that converts GBA-V XML responses to HaalCentraal JSON format.
- `lib`: Contains multiple libraries and protocols that are shared between the `wallet` and other applications.
- `demo_relying_party`: Demo Pelying Party server, mocks multiple use cases.
- `tests_integration`: Integration tests for the `wallet` and core applications.
- `uniffi-bindgen`: Helpers for `wallet/platform_support` bridge code generation.
- `update_policy`: Server component for the update policy and shared data types with `wallet`.
- `wallet`: Contains the wallet business logic, i.e. the main crate, and related subcrates.
  - `platform_support`: Contains native functionality for both Android and iOS and code to bridge to these platforms.
- `wallet_ca`: CLI to generate issuer and reader certificates for local development.
- `wallet_provider`: The Wallet Provider server, which contains the Account Server.
  - `wallet_account`: Code shared between `wallet` and `wallet_provider`.
- `wallet_server`: VV/OV helper servers for issuers (`pid_issuer`) and/or verifiers (`verification_server`)

## Error types

Because of the different contexts in which each of these crates operate, error handling has been implemented according to the needs of these context.

As the `wallet_account` crate provides a library of functionality to both `wallet` and `wallet_provider`, all errors have been consolidated into a single Error type, i.e. `wallet_account::error::Error`.
For convenience, a `wallet_account::error::Result` type is also provided.

The `platform_support` crate also acts as a library to the `wallet` crate, however is functionality is separated into distinct modules.
Each of these modules provide their own error type.

The `wallet` and `wallet_provider` crates provide the main entry points into the business logic of the app and Wallet Provider respectively.
As such, more finely grained error types are provided, with each method on the `Wallet` and `AccountServer` types having their own error type defined.
This allows for detailed error reporting to Flutter in case of `Wallet` and specific error codes and HTTP(S) response codes in case of `AccountServer`.
Additionally, the `wallet` crate has some error types provided for internal functionality.

## Instructions

### Regenerate the flutter bindings

To regenerate the bindings, run the following command from the root:

```
cargo install flutter_rust_bridge_codegen@2.8.0 && \
flutter_rust_bridge_codegen generate --config-file wallet_app/flutter_rust_bridge.yaml
```

## Code Conventions

### Imports

In our rust files, we order the imports (`use` statements) first by the following categories, separated by a newline, and then alphabetically.

1. Standard Library imports
2. 3pp (Third-Party Package) imports
3. Workspace imports
4. Local imports

```rs
// Standard Library imports
use std::*;

// 3pp imports
use serde::...;

// Workspace imports
use wallet_account::...;

// Local imports
use crate::...;
use super::...;
```

## Tests

The tests can be run with the normal cargo test runner,
but also with the [nextest](https://nexte.st) runner.
Install nextest via:

```sh
cargo install cargo-nextest --locked
```

and run via `cargo nextest run` instead of `cargo test`.
