# Regenerate the flutter bindings

To regenerate the bindings, run the following command from `wallet_core`:

```sh
cargo run --manifest-path flutter_rust_bridge_codegen/Cargo.toml
```

# Regenerate the datatypes for bincode

To regenerate the data types, run the following command from `wallet_core`:

```sh
cargo run --bin serde_reflection_codegen --features serde_reflection_codegen
```

This will generate dart code in `$PROJECT_ROOT/pub/core_domain`.
After which the classes are available in the Flutter app by importing `import 'package:core_domain/core_domain.dart';`.

# Project structure

- `wallet`: Contains the wallet business logic
- `flutter_api`: Contains the `api.rs` for `flutter_rust_bridge` and the data types for the API
