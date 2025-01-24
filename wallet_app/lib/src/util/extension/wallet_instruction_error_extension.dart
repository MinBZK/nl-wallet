import 'package:wallet_core/core.dart';

import '../../domain/usecase/pin/check_pin_usecase.dart';

extension WalletInstructionErrorExtension on WalletInstructionError {
  CheckPinResult asCheckPinResult() {
    return map<CheckPinResult>(
      incorrectPin: (result) => CheckPinResultIncorrect(
        attemptsLeftInRound: result.attemptsLeftInRound,
        isFinalRound: result.isFinalRound,
      ),
      timeout: (result) => CheckPinResultTimeout(timeoutMillis: result.timeoutMillis.toInt()),
      blocked: (result) => CheckPinResultBlocked(),
    );
  }
}
