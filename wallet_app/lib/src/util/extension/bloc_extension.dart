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
  ///
  /// Note: Make sure to await this method if you are emitting from the callbacks.
  Future<void> handleError(
    Object ex, {
    Function(CoreGenericError)? onGenericError,
    //ignore: avoid_positional_boolean_parameters
    Function(CoreNetworkError, bool /* hasInternet */)? onNetworkError,
    Function(CoreRedirectUriError)? onRedirectUriError,
    Function(CoreHardwareKeyUnsupportedError)? onHardwareKeyUnsupportedError,
    Function(CoreDisclosureSourceMismatchError)? onDisclosureSourceMismatchError,
    Function(CoreExpiredSessionError)? onCoreExpiredSessionError,
    Function(CoreCancelledSessionError)? onCoreCancelledSessionError,
    Function(CoreError)? onCoreError,
    required Function(Object) onUnhandledError,
  }) async {
    if (ex is CoreError) {
      switch (ex) {
        case CoreGenericError():
          await onGenericError?.call(ex);
          if (onGenericError != null) return;
        case CoreNetworkError():
          await onNetworkError?.call(ex, await checkHasInternetUseCase.invoke());
          if (onNetworkError != null) return;
        case CoreRedirectUriError():
          await onRedirectUriError?.call(ex);
          if (onRedirectUriError != null) return;
        case CoreHardwareKeyUnsupportedError():
          await onHardwareKeyUnsupportedError?.call(ex);
          if (onHardwareKeyUnsupportedError != null) return;
        case CoreDisclosureSourceMismatchError():
          await onDisclosureSourceMismatchError?.call(ex);
          if (onDisclosureSourceMismatchError != null) return;
        case CoreExpiredSessionError():
          await onCoreExpiredSessionError?.call(ex);
          if (onCoreExpiredSessionError != null) return;
        case CoreCancelledSessionError():
          await onCoreCancelledSessionError?.call(ex);
          if (onCoreCancelledSessionError != null) return;
        case CoreStateError():
          // This is a programming error and thus should not be handled gracefully.
          throw ex;
      }
      await onCoreError?.call(ex);
      if (onCoreError != null) return;
    }
    await onUnhandledError.call(ex);
  }
}
