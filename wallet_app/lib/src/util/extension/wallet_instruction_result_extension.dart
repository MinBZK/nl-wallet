import '../../../bridge_generated.dart';
import '../../domain/usecase/pin/check_pin_usecase.dart';

extension WalletInstructionResultExtension on WalletInstructionResult {
  CheckPinResult asCheckPinResult() {
    return map<CheckPinResult>(
      ok: (result) => CheckPinResultOk(),
      incorrectPin: (result) => CheckPinResultIncorrect(
        leftoverAttempts: result.leftoverAttempts,
        isFinalAttempt: result.isFinalAttempt,
      ),
      timeout: (result) => CheckPinResultTimeout(timeoutMillis: result.timeoutMillis),
      blocked: (result) => CheckPinResultBlocked(),
    );
  }
}
