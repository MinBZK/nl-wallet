import '../../../rust_core.dart';

class CheckIsValidPinUseCase {
  final RustCore _rustCore;

  CheckIsValidPinUseCase(this._rustCore);

  Future<bool> invoke(String pin) async => _rustCore.isValidPin(pin: pin);
}
