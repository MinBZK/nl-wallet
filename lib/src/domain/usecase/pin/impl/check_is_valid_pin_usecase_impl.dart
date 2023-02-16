import 'package:core_domain/core_domain.dart';
import 'package:fimber/fimber.dart';

import '../../../../rust_core.dart';
import '../check_is_valid_pin_usecase.dart';

class CheckIsValidPinUseCaseImpl implements CheckIsValidPinUseCase {
  final RustCore _rustCore;

  CheckIsValidPinUseCaseImpl(this._rustCore);

  @override
  Future<bool> invoke(String pin) async {
    final bytes = await _rustCore.isValidPin(pin: pin);
    final result = PinResult.bincodeDeserialize(bytes);
    if (result is PinResultErrItem) {
      Fimber.d('Pin considered too simple. Reason: ${result.value}');
    }
    return result is PinResultOkItem;
  }
}
