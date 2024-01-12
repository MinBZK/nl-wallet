import 'package:wallet_core/core.dart';

import '../../domain/usecase/pin/check_pin_usecase.dart';
import 'wallet_instruction_error_extension.dart';

extension AcceptDisclosureResultExtension on AcceptDisclosureResult {
  CheckPinResult asCheckPinResult() {
    return map<CheckPinResult>(
      ok: (result) => CheckPinResultOk(returnUrl: result.returnUrl),
      instructionError: (result) => result.error.asCheckPinResult(),
    );
  }
}
