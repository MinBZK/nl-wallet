import 'package:core_domain/core_domain.dart';

import '../../../domain/model/pin/pin_validation_error.dart';
import '../mapper.dart';

class PinValidationErrorMapper extends Mapper<PinError, PinValidationError> {
  @override
  PinValidationError map(PinError input) {
    switch (input) {
      case PinError.nonDigits:
      case PinError.invalidLength:
        return PinValidationError.other;
      case PinError.tooLittleUniqueDigits:
        return PinValidationError.tooLittleUniqueDigits;
      case PinError.ascendingDigits:
      case PinError.descendingDigits:
        return PinValidationError.sequentialDigits;
    }
  }
}
