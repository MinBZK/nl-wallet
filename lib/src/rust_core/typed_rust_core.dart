import 'package:core_domain/core_domain.dart';

import 'rust_core.dart';

/// Wraps the generated RustCore to provide
/// a typed interface using the SERDE generated
/// models from the 'core_domain' package.
class TypedRustCore {
  final RustCore _rustCore;

  TypedRustCore(this._rustCore) {
    // Initialize the Asynchronous runtime of the Rust core module.
    // This is required to be able to execute asynchronous Rust functions.
    _rustCore.initAsync();
  }

  Future<PinValidationResult> isValidPin(String pin) async {
    final bytes = await _rustCore.isValidPin(pin: pin);
    return PinValidationResultExtension.bincodeDeserialize(bytes);
  }

  Future<void> register(String pin) async {
    await _rustCore.register(pin: pin);
  }
}
