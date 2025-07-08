import 'package:test/test.dart';
import 'package:wallet_core/core.dart';
import 'package:wallet_mock/src/pin/pin_manager.dart';

const kTestValidPin = '909090';
const kTestInvalidPin = '112233';

void main() {
  late PinManager pinManager;
  setUp(() {
    pinManager = PinManager();
  });

  group('registration', () {
    test('pin is not registered at creation', () {
      expect(pinManager.isRegistered, isFalse);
    });

    test('pin is registered after setPin', () {
      pinManager.setPin(kTestValidPin);
      expect(pinManager.isRegistered, isTrue);
    });

    test('pin can not be registered twice', () {
      pinManager.setPin(kTestValidPin);
      expect(() => pinManager.setPin(kTestValidPin), throwsA(const TypeMatcher<StateError>()));
    });
  });

  group('reset', () {
    test('calling reset pin resets the isRegistered flag', () {
      expect(pinManager.isRegistered, isFalse);
      pinManager.setPin(kTestValidPin);
      expect(pinManager.isRegistered, isTrue);
      pinManager.resetPin();
      expect(pinManager.isRegistered, isFalse);
    });
  });

  group('check pin', () {
    test('checking pin before registration throws', () {
      expect(() => pinManager.checkPin(kTestValidPin), throwsA(const TypeMatcher<StateError>()));
    });

    test('checking the correct pin results in ok', () {
      pinManager.setPin(kTestValidPin);
      expect(pinManager.checkPin(kTestValidPin), const WalletInstructionResult.ok());
    });

    test('checking an incorrect pin results in incorrectPin', () {
      pinManager.setPin(kTestValidPin);

      const incorrectPin = WalletInstructionError.incorrectPin(attemptsLeftInRound: 3, isFinalRound: false);
      expect(
        pinManager.checkPin(kTestInvalidPin),
        const WalletInstructionResult.instructionError(error: incorrectPin),
      );
    });

    test('checking an incorrect pin 3 times results in timeout', () {
      pinManager.setPin(kTestValidPin);
      pinManager.checkPin(kTestInvalidPin);
      pinManager.checkPin(kTestInvalidPin);
      pinManager.checkPin(kTestInvalidPin);

      final result = pinManager.checkPin(kTestInvalidPin);
      expect((result as WalletInstructionResult_InstructionError).error, isA<WalletInstructionError_Timeout>());
    });

    test('checking an incorrect pin 12 times results in blocked', () {
      pinManager.setPin(kTestValidPin);
      int i = 0;
      while (i < 11) {
        pinManager.checkPin(kTestInvalidPin);
        i++;
      }

      final result = pinManager.checkPin(kTestInvalidPin);
      expect((result as WalletInstructionResult_InstructionError).error, isA<WalletInstructionError_Blocked>());
    });

    test('checking an incorrect pin 8 and then checking a correct pin results in ok', () {
      pinManager.setPin(kTestValidPin);
      int i = 0;
      while (i < 8) {
        pinManager.checkPin(kTestInvalidPin);
        i++;
      }
      expect(pinManager.checkPin(kTestValidPin), const WalletInstructionResult.ok());
    });

    test('checking an incorrect pin 11 reports to the user that it is the last attempt', () {
      pinManager.setPin(kTestValidPin);
      int i = 0;
      while (i < 10) {
        pinManager.checkPin(kTestInvalidPin);
        i++;
      }

      const expected = WalletInstructionResult.instructionError(
        error: WalletInstructionError.incorrectPin(
          attemptsLeftInRound: 1,
          isFinalRound: true,
        ),
      );
      final result = pinManager.checkPin(kTestInvalidPin);
      expect(result, expected);
    });

    test('checking the correct pin after the wallet is already blocked results in blocked', () {
      pinManager.setPin(kTestValidPin);
      int i = 0;
      while (i < 12) {
        pinManager.checkPin(kTestInvalidPin);
        i++;
      }
      const expected = WalletInstructionResult.instructionError(
        error: WalletInstructionError.blocked(),
      );
      expect(pinManager.checkPin(kTestValidPin), expected);
    });
  });
}
