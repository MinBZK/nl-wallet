import 'package:wallet_core/core.dart';

import '../../domain/model/result/application_error.dart';
import '../../domain/model/result/result.dart';
import 'wallet_instruction_error_extension.dart';

extension WalletInstructionResultExtension on WalletInstructionResult {
  Result<String?> asApplicationResult() {
    return switch (this) {
      WalletInstructionResult_Ok() => Result.success(null),
      WalletInstructionResult_InstructionError(:final error) => Result.error(
          CheckPinError(
            error.asCheckPinResult(),
            sourceError: this,
          ),
        ),
    };
  }
}
