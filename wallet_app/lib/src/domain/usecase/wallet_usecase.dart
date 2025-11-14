import 'dart:io';

import 'package:fimber/fimber.dart';
import 'package:wallet_core/core.dart';

import '../../util/extension/core_error_extension.dart';
import '../../util/extension/wallet_instruction_error_extension.dart';
import '../../wallet_core/error/core_error.dart';
import '../model/result/application_error.dart';
import '../model/result/result.dart';

abstract class WalletUseCase {
  /// Usecase helper method that implements default error handling.
  ///
  /// This method catches any thrown exceptions and makes sure they
  /// are exposed as an [ApplicationError]. When more fine-grained error handling
  /// is needed usecases can implement a similar pattern without this function.
  Future<Result<T>> tryCatch<T>(Future<T> Function() future, String errorDescription) async {
    try {
      return Result.success(await future());
    } catch (ex) {
      Fimber.e(errorDescription, ex: ex);
      return Result.error(await exceptionToApplicationError(ex));
    }
  }

  static Future<ApplicationError> exceptionToApplicationError(Object ex) async {
    if (ex is ApplicationError) {
      return ex;
    } else if (ex is CoreError) {
      return ex.asApplicationError();
    } else if (ex is WalletInstructionError) {
      final checkPinResult = ex.asCheckPinResult();
      return CheckPinError(checkPinResult, sourceError: ex);
    } else if (ex is StateError) {
      Fimber.e('StateErrors indicate programming errors and should not be handled gracefully', ex: ex);
      exit(1);
    }
    return GenericError(ex.toString(), sourceError: ex);
  }
}

extension WalletUseCaseStreamExtension<T> on Stream<T> {
  /// Helper method for Stream exposing usecases. This method makes sure any errors exposed by
  /// the source stream are converted into [ApplicationError]s.
  /// Note: do not use for streams that might emit after the listener has cancelled the subscription.
  Stream<T> handleAppError(String errorMessage) async* {
    try {
      await for (final value in this) {
        yield value;
      }
    } catch (ex) {
      Fimber.e(errorMessage, ex: ex);
      throw (await WalletUseCase.exceptionToApplicationError(ex));
    }
  }
}
