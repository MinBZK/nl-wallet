import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/forgot_pin/argument/forgot_pin_screen_argument.dart';

void main() {
  group('ForgotPinScreenArgument', () {
    test('should be correctly instantiated', () {
      const argument = ForgotPinScreenArgument(useCloseButton: true);
      expect(argument.useCloseButton, isTrue);
    });

    test('should support equality', () {
      const argument1 = ForgotPinScreenArgument(useCloseButton: true);
      const argument2 = ForgotPinScreenArgument(useCloseButton: true);
      const argument3 = ForgotPinScreenArgument(useCloseButton: false);

      expect(argument1, equals(argument2));
      expect(argument1, isNot(equals(argument3)));
    });

    test('should support JSON serialization', () {
      const argument = ForgotPinScreenArgument(useCloseButton: true);
      final json = argument.toJson();
      final fromJson = ForgotPinScreenArgument.fromJson(json);

      expect(fromJson, equals(argument));
      expect(json['useCloseButton'], isTrue);
    });
  });
}
