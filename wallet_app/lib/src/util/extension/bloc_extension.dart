import 'package:flutter_bloc/flutter_bloc.dart';

import '../../wallet_core/error/flutter_api_error.dart';

extension BlocExtensions on Bloc {
  /// This is a convenience method to process caught exceptions in a [Bloc].
  /// Callbacks can be provided to handle a more concrete type of the exception.
  /// Only one callback is ever fired and it's always the most specific callback
  /// that is called. As a fallback you always need to provide an [onUnhandledError]
  /// callback to make sure no exception goes uncaught.
  void handleError(
    Object ex, {
    Function(FlutterApiError)? onGenericError,
    Function(FlutterApiError)? onNetworkError,
    Function(FlutterApiError)? onFlutterApiError,
    required Function(Object) onUnhandledError,
  }) {
    if (ex is FlutterApiError) {
      switch (ex.type) {
        case FlutterApiErrorType.generic:
          if (onGenericError != null) {
            onGenericError.call(ex);
            return;
          }
        case FlutterApiErrorType.networking:
          if (onNetworkError != null) {
            onNetworkError.call(ex);
            return;
          }
      }
      if (onFlutterApiError != null) {
        onFlutterApiError.call(ex);
        return;
      }
    }
    onUnhandledError.call(ex);
  }
}
