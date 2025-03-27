import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/pin/check_pin_result.dart';
import 'package:wallet/src/util/extension/wallet_instruction_error_extension.dart';
import 'package:wallet_core/core.dart';

void main() {
  test('WalletInstructionError.incorrectPin is converted to CheckPinResultIncorrect', () async {
    final input = WalletInstructionError.incorrectPin(attemptsLeftInRound: 3, isFinalRound: true);
    final result = input.asCheckPinResult() as CheckPinResultIncorrect;
    expect(result.isFinalRound, isTrue);
    expect(result.attemptsLeftInRound, 3);
  });

  test('WalletInstructionError.timeout is converted to CheckPinResultTimeout', () async {
    final input = WalletInstructionError.timeout(timeoutMillis: BigInt.from(12345));
    final result = input.asCheckPinResult() as CheckPinResultTimeout;
    expect(result.timeoutMillis, 12345);
  });

  test('WalletInstructionError.blocked is converted to CheckPinResultIncorrect', () async {
    final input = WalletInstructionError.blocked();
    final result = input.asCheckPinResult();
    expect(result is CheckPinResultBlocked, isTrue);
  });
}
