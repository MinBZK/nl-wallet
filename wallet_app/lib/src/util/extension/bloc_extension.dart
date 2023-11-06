import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/usecase/network/check_has_internet_usecase.dart';
import '../../wallet_core/error/core_error.dart';

extension BlocExtensions on Bloc {
  /// Static reference, set on app start, needed to check
  /// internet connection from the [handleError] method.
  static late CheckHasInternetUseCase checkHasInternetUseCase;

  /// This is a convenience method to process caught exceptions in a [Bloc].
  /// Callbacks can be provided to handle a more concrete type of the exception.
  /// Only one callback is ever fired and it's always the most specific callback
  /// that is called. As a fallback you always need to provide an [onUnhandledError]
  /// callback to make sure no exception goes uncaught.
  Future<void> handleError(
    Object ex, {
    Function(CoreGenericError)? onGenericError,
    Function(CoreNetworkError, bool /* hasInternet */)? onNetworkError,
    Function(CoreRedirectUriError)? onRedirectUriError,
    Function(CoreError)? onCoreError,
    required Function(Object) onUnhandledError,
  }) async {
    if (ex is CoreError) {
      switch (ex) {
        case CoreGenericError():
          if (onGenericError != null) {
            onGenericError.call(ex);
            return;
          }
        case CoreNetworkError():
          if (onNetworkError != null) {
            onNetworkError.call(ex, await checkHasInternetUseCase.invoke());
            return;
          }
        case CoreRedirectUriError():
          if (onRedirectUriError != null) {
            onRedirectUriError.call(ex);
            return;
          }
        case CoreStateError():
          // This is a programming error and thus should not be handled gracefully.
          throw ex;
      }
      if (onCoreError != null) {
        onCoreError.call(ex);
        return;
      }
    }
    onUnhandledError.call(ex);
  }
}
