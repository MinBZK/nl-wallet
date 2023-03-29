import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/data/repository/wallet/mock/mock_wallet_repository.dart';
import 'package:wallet/src/data/source/memory/memory_wallet_datasource.dart';
import 'package:wallet/src/domain/model/pin/pin_validation_error.dart';
import 'package:wallet/src/domain/usecase/pin/check_is_valid_pin_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/impl/check_is_valid_pin_usecase_impl.dart';

void main() {
  //TODO: Replace with proper mocks
  final walletDataSource = MemoryWalletDataSource();
  final walletRepository = MockWalletRepository(walletDataSource);

  late CheckIsValidPinUseCase useCase;

  setUp(() {
    useCase = CheckIsValidPinUseCaseImpl(walletRepository);
  });

  test('should not throw when valid pin is provided', () async {
    try {
      await useCase.invoke('133700');
    } catch (error) {
      expect(error, null);
    }
  });

  test('should throw a PinValidationError.other when a pin with less than 6 digits is provided', () async {
    try {
      await useCase.invoke('123');
    } catch (error) {
      expect(error, PinValidationError.other);
    }
  });

  test('should throw a PinValidationError.sequentialDigits error when 123456 is provided as a pin', () async {
    try {
      await useCase.invoke('123456');
    } catch (error) {
      expect(error, PinValidationError.sequentialDigits);
    }
  });

  test('should throw a PinValidationError.tooFewUniqueDigits error when 555555 is provided as a pin', () async {
    try {
      await useCase.invoke('555555');
    } catch (error) {
      expect(error, PinValidationError.tooFewUniqueDigits);
    }
  });
}
