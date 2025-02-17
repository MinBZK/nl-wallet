import 'package:wallet_core/core.dart';

import '../../domain/model/result/application_error.dart';
import '../../domain/model/result/result.dart';
import 'wallet_instruction_error_extension.dart';

extension WalletInstructionResultExtension on WalletInstructionResult {
  Result<String?> asApplicationResult() {
    return map(
      ok: (result) => const Result.success(null),
      instructionError: (error) => Result.error(
        IncorrectPinError(error.error.asCheckPinResult(), sourceError: error),
      ),
    );
  }
}
