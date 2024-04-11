import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:sentry_flutter/sentry_flutter.dart';

class WalletErrorHandler {
  /// Return true to indicate the exception has been handled
  bool handlerError(Object error, StackTrace stack) {
    FlutterError.presentError(FlutterErrorDetails(exception: error, stack: stack));
    Sentry.captureException(error, stackTrace: stack).then((value) => exit(1));
    return true;
  }
}
