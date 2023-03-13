# Regenerate the flutter bindings

To regenerate the bindings, run the following command from the root of the repository:

```sh
cargo run --manifest-path flutter_rust_bridge_codegen/Cargo.toml
```

# Regenerate the datatypes for bincode

From the folder `rust_core/flutter-data-types`, run the following command:

```sh
cargo run
```

This will generate dart code in `$PROJECT_ROOT/pub/core_domain`. After which the classes are available in the Flutter app by importing `import 'package:core_domain/core_domain.dart';
`.

# Project structure

- `rust_core`: Contains the `api.rs` for `flutter_rust_bridge`, and implementations.
- `rust_core/flutter-data-types`: Contains a generator for the data types.
