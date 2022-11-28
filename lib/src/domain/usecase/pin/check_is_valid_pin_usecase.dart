import 'package:flutter/widgets.dart';

import '../../../wallet_constants.dart';

class CheckIsValidPinUseCase {
  CheckIsValidPinUseCase();

  bool invoke(String pin) {
    if (pin.characters.toSet().length <= 1) return false;
    if (pin == '123456') return false;
    if (pin == '654321') return false;
    return pin.length == kPinDigits;
  }
}
