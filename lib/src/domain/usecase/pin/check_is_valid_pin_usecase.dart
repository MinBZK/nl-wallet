import 'package:flutter/widgets.dart';

import '../../../wallet_constants.dart';

class CheckIsValidPinUseCase {
  CheckIsValidPinUseCase();

  bool invoke(String pin) => pin.characters.toSet().length > 1 && pin.length == kPinDigits;
}
