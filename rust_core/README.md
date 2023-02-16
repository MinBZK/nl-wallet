# Regenerate the flutter bindings

This assumes that the `flutter_rust_bridge_codegen` is installed with:

```sh
cargo install flutter_rust_bridge_codegen
```

To regenerate the bindings, run the following command from the root of the repository:

```sh
flutter_rust_bridge_codegen \
    --rust-input rust_core/src/api.rs \
    --dart-output lib/bridge_generated.dart \
    --c-output ios/Runner/bridge_generated.h \
    --rust-output rust_core/src/bridge_generated/bridge.rs \
    --skip-add-mod-to-lib
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
