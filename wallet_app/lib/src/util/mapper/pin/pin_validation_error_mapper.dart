import 'package:wallet_core/core.dart';

import '../../../domain/model/pin/pin_validation_error.dart';
import '../../../util/mapper/mapper.dart';

class PinValidationErrorMapper extends Mapper<PinValidationResult, PinValidationError?> {
  @override
  PinValidationError? map(PinValidationResult input) {
    switch (input) {
      case PinValidationResult.Ok:
        return null;
      case PinValidationResult.TooFewUniqueDigits:
        return PinValidationError.tooFewUniqueDigits;
      case PinValidationResult.SequentialDigits:
        return PinValidationError.sequentialDigits;
      case PinValidationResult.OtherIssue:
        return PinValidationError.other;
    }
  }
}
