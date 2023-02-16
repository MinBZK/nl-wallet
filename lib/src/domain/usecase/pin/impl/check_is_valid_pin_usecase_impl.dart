import '../../../../rust_core.dart';
import '../check_is_valid_pin_usecase.dart';

class CheckIsValidPinUseCaseImpl implements CheckIsValidPinUseCase {
  final RustCore _rustCore;

  CheckIsValidPinUseCaseImpl(this._rustCore);

  @override
  Future<bool> invoke(String pin) async => _rustCore.isValidPin(pin: pin);
}
