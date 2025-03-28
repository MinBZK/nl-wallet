import 'package:wallet_core/core.dart';

import '../../domain/model/pin/check_pin_result.dart';

extension WalletInstructionErrorExtension on WalletInstructionError {
  CheckPinResult asCheckPinResult() {
    return switch (this) {
      WalletInstructionError_IncorrectPin(:final attemptsLeftInRound, :final isFinalRound) =>
        CheckPinResultIncorrect(attemptsLeftInRound: attemptsLeftInRound, isFinalRound: isFinalRound),
      WalletInstructionError_Timeout(:final timeoutMillis) =>
        CheckPinResultTimeout(timeoutMillis: timeoutMillis.toInt()),
      WalletInstructionError_Blocked() => CheckPinResultBlocked(),
    };
  }
}
