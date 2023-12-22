import 'package:wallet_core/core.dart';

import '../../domain/usecase/pin/check_pin_usecase.dart';

extension WalletInstructionErrorExtension on WalletInstructionError {
  CheckPinResult asCheckPinResult() {
    return map<CheckPinResult>(
      incorrectPin: (result) => CheckPinResultIncorrect(
        leftoverAttempts: result.leftoverAttempts,
        isFinalAttempt: result.isFinalAttempt,
      ),
      timeout: (result) => CheckPinResultTimeout(timeoutMillis: result.timeoutMillis),
      blocked: (result) => CheckPinResultBlocked(),
    );
  }
}
