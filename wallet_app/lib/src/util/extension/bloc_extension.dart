import 'package:flutter_bloc/flutter_bloc.dart';

import '../../wallet_core/error/core_error.dart';

extension BlocExtensions on Bloc {
  /// This is a convenience method to process caught exceptions in a [Bloc].
  /// Callbacks can be provided to handle a more concrete type of the exception.
  /// Only one callback is ever fired and it's always the most specific callback
  /// that is called. As a fallback you always need to provide an [onUnhandledError]
  /// callback to make sure no exception goes uncaught.
  void handleError(
    Object ex, {
    Function(CoreGenericError)? onGenericError,
    Function(CoreNetworkError)? onNetworkError,
    Function(CoreRedirectUriError)? onRedirectUriError,
    Function(CoreError)? onCoreError,
    required Function(Object) onUnhandledError,
  }) {
    if (ex is CoreError) {
      switch (ex) {
        case CoreGenericError():
          if (onGenericError != null) {
            onGenericError.call(ex);
            return;
          }
        case CoreNetworkError():
          if (onNetworkError != null) {
            onNetworkError.call(ex);
            return;
          }
        case CoreRedirectUriError():
          if (onRedirectUriError != null) {
            onRedirectUriError.call(ex);
            return;
          }
      }
      if (onCoreError != null) {
        onCoreError.call(ex);
        return;
      }
    }
    onUnhandledError.call(ex);
  }
}
