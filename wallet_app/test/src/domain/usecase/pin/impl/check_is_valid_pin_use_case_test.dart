import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/pin/pin_validation_error.dart';
import 'package:wallet/src/domain/usecase/pin/check_is_valid_pin_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/impl/check_is_valid_pin_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockWalletRepository walletRepository;

  late CheckIsValidPinUseCase useCase;

  setUp(() {
    walletRepository = MockWalletRepository();
    // Set up default
    useCase = CheckIsValidPinUseCaseImpl(walletRepository);
  });

  test('should not throw when valid pin is provided', () async {
    try {
      const validPin = '133700';
      when(walletRepository.validatePin(validPin)).thenAnswer((_) async {});
      await useCase.invoke(validPin);
    } catch (error) {
      expect(error, null);
    }
  });

  test('should throw a PinValidationError.other when a pin with less than 6 digits is provided', () async {
    try {
      const shorPin = '123';
      when(walletRepository.validatePin(shorPin)).thenAnswer((_) => throw PinValidationError.other);
      await useCase.invoke(shorPin);
      assert(false, 'unreachable');
    } catch (error) {
      expect(error, PinValidationError.other);
    }
  });

  test('should throw a PinValidationError.sequentialDigits error when 123456 is provided as a pin', () async {
    try {
      const sequentialPin = '123456';
      when(walletRepository.validatePin(sequentialPin)).thenAnswer((_) => throw PinValidationError.sequentialDigits);
      await useCase.invoke(sequentialPin);
      assert(false, 'unreachable');
    } catch (error) {
      expect(error, PinValidationError.sequentialDigits);
    }
  });

  test('should throw a PinValidationError.tooFewUniqueDigits error when 555555 is provided as a pin', () async {
    try {
      const nonUniquePin = '555555';
      when(walletRepository.validatePin(nonUniquePin)).thenAnswer((_) => throw PinValidationError.tooFewUniqueDigits);
      await useCase.invoke(nonUniquePin);
      assert(false, 'unreachable');
    } catch (error) {
      expect(error, PinValidationError.tooFewUniqueDigits);
    }
  });
}
