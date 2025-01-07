# Wallet core

This contains the "shared core" part of the app.

## Project structure

- `wallet`: Contains the wallet business logic, i.e. the main crate.
- `wallet_provider`: The Wallet Provider server, which contains the Account Server.
- `wallet_common`: Code shared between `wallet` and `wallet_provider`.
- `flutter_api`: Contains the `api.rs` for `flutter_rust_bridge` and the data types for the API. This allows Flutter to use the functionality from the `wallet` crate.
- `platform_support`: Contains native functionality for both Android and iOS and code to bridge to these platforms.
- `flutter_rust_bridge_codegen` & `uniffi-bindgen`: Helpers for bridge code generation.

## Error types

Because of the different contexts in which each of these crates operate, error handling has been implemented according to the needs of these context.

As the `wallet_common` crate provides a library of functionality to both `wallet` and `wallet_provider`, all errors have been consolidated into a single Error type, i.e. `wallet_common::errors::Error`.
For convenience, a `wallet_common::errors:Result` type is also provided.

The `platform_support` crate also acts as a library to the `wallet` crate, however is functionality is separated into distinct modules.
Each of these modules provide their own error type.

The `wallet` and `wallet_provider` crates provide the main entry points into the business logic of the app and Wallet Provider respectively.
As such, more finely grained error types are provided, with each method on the `Wallet` and `AccountServer` types having their own error type defined.
This allows for detailed error reporting to Flutter in case of `Wallet` and specific error codes and HTTP(S) response codes in case of `AccountServer`.
Additionally, the `wallet` crate has some error types provided for internal functionality.

## Instructions

### Regenerate the flutter bindings

Generating the flutter bindings requires installing the `cargo-expand` utility:

```sh
cargo install cargo-expand
```

To regenerate the bindings, run the following command from `wallet_core`:

```sh
cargo run --manifest-path flutter_rust_bridge_codegen/Cargo.toml
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
use wallet_common::...;

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
