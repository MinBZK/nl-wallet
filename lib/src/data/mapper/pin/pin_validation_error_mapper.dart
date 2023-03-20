import 'package:core_domain/core_domain.dart';

import '../../../domain/model/pin/pin_validation_error.dart';
import '../mapper.dart';

class PinValidationErrorMapper extends Mapper<PinResult, PinValidationError?> {
  @override
  PinValidationError? map(PinResult input) {
    switch (input) {
      case PinResult.ok:
        return null;
      case PinResult.tooFewUniqueDigitsError:
        return PinValidationError.tooFewUniqueDigits;
      case PinResult.sequentialDigitsError:
        return PinValidationError.sequentialDigits;
      case PinResult.otherError:
        return PinValidationError.other;
    }
  }
}
