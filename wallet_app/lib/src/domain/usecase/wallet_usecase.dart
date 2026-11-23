import 'dart:async';

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
    }
    // Note: StateErrors (invariant violations / programming errors) used to crash the app here via
    // exit(1). They now fall through to a GenericError; the invariant error screen is driven by the
    // core error listener and the global WalletErrorHandler instead.
    return GenericError(ex.toString(), sourceError: ex);
  }
}

extension WalletUseCaseStreamExtension<T> on Stream<T> {
  /// Helper method for Stream exposing usecases. This method makes sure any errors exposed by
  /// the source stream are converted into [ApplicationError]s. The stream closes after the first error.
  Stream<T> handleAppError(String errorMessage) {
    // Deliberately not an async* generator: one suspended in `await for` keeps listening to the source
    // until its next event, so upstream work continues after cancel and its errors escape through the
    // (often unawaited) cancel future.
    StreamSubscription<T>? subscription;
    late final StreamController<T> controller;
    controller = StreamController<T>(
      sync: true,
      onListen: () {
        subscription = listen(
          controller.add,
          onError: (Object ex, StackTrace stackTrace) async {
            // Cleared so the consumer's later cancel doesn't get this same (possibly failing) cancel future.
            final source = subscription;
            subscription = null;
            // Not awaited: a cancel that fails or never completes must not keep the error from the consumer.
            unawaited(
              source?.cancel().catchError(
                (Object cancelEx, StackTrace cancelStackTrace) =>
                    Fimber.w('Failed to cancel source after error', ex: cancelEx, stacktrace: cancelStackTrace),
              ),
            );
            Fimber.e(errorMessage, ex: ex, stacktrace: stackTrace);
            try {
              controller.addError(await WalletUseCase.exceptionToApplicationError(ex), stackTrace);
            } catch (mapEx, mapStackTrace) {
              // Mapping can fail (e.g. the internet check for a CoreNetworkError); consumers still expect an
              // ApplicationError and a closed stream.
              Fimber.e('Failed to map source error', ex: mapEx, stacktrace: mapStackTrace);
              controller.addError(GenericError(ex.toString(), sourceError: ex), stackTrace);
            } finally {
              await controller.close();
            }
          },
          onDone: controller.close,
        );
      },
      onPause: () => subscription?.pause(),
      onResume: () => subscription?.resume(),
      onCancel: () => subscription?.cancel(),
    );
    return controller.stream;
  }
}
