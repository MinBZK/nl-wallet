import 'package:fimber/fimber.dart';

import '../../util/extension/core_error_extension.dart';
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
    } on ApplicationError catch (ex) {
      Fimber.e(errorDescription, ex: ex);
      return Result.error(ex);
    } on CoreError catch (ex) {
      Fimber.e(errorDescription, ex: ex);
      return Result.error(await ex.asApplicationError());
    } catch (ex) {
      Fimber.e(errorDescription, ex: ex);
      return Result.error(GenericError(ex.toString(), sourceError: ex));
    }
  }
}

extension WalletUseCaseStreamExtension<T> on Stream<T> {
  /// Helper method for Stream exposing usecases. This method makes sure any errors exposed by
  /// the source stream are converted into [ApplicationError]s.
  Stream<T> handleAppError(String errorMessage) async* {
    try {
      await for (final value in this) {
        yield value;
      }
    } on ApplicationError catch (ex) {
      Fimber.e(errorMessage, ex: ex);
      rethrow;
    } on CoreError catch (ex) {
      Fimber.e(errorMessage, ex: ex);
      throw await ex.asApplicationError();
    } catch (ex) {
      Fimber.e(errorMessage, ex: ex);
      throw GenericError(ex.toString(), sourceError: ex);
    }
  }
}
