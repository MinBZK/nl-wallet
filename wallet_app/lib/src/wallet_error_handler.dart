import 'dart:io';

import 'package:flutter/foundation.dart';

class WalletErrorHandler {
  /// Return true to indicate the exception has been handled
  bool handlerError(Object error, StackTrace stack) {
    FlutterError.presentError(FlutterErrorDetails(exception: error, stack: stack));
    exit(1);
  }
}
