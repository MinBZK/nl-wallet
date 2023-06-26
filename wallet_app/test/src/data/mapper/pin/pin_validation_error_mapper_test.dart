import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/bridge_generated.dart';
import 'package:wallet/src/data/mapper/pin/pin_validation_error_mapper.dart';
import 'package:wallet/src/domain/model/pin/pin_validation_error.dart';

void main() {
  final mapper = PinValidationErrorMapper();

  test('PinValidationErrorMapper maps to the expected values', () {
    expect(mapper.map(PinValidationResult.Ok), null);
    expect(mapper.map(PinValidationResult.TooFewUniqueDigits), PinValidationError.tooFewUniqueDigits);
    expect(mapper.map(PinValidationResult.SequentialDigits), PinValidationError.sequentialDigits);
    expect(mapper.map(PinValidationResult.OtherIssue), PinValidationError.other);
  });
}
