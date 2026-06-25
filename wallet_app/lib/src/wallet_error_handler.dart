import 'dart:async';

import 'package:flutter/material.dart';
import 'package:sentry_flutter/sentry_flutter.dart';

import 'feature/error/invariant/argument/invariant_error_screen_argument.dart';
import 'navigation/wallet_routes.dart';

/// Handles otherwise uncaught errors. Instead of crashing the app (the previous behaviour), these are
/// reported to Sentry and the user is navigated to the invariant error screen.
class WalletErrorHandler {
  /// Key that holds the [NavigatorState], used to navigate from this non-Widget context.
  final GlobalKey<NavigatorState> _navigatorKey;

  WalletErrorHandler(this._navigatorKey);

  /// Return true to indicate the exception has been handled
  bool handleError(Object error, StackTrace stack) {
    FlutterError.presentError(FlutterErrorDetails(exception: error, stack: stack));
    unawaited(_reportFatalDartErrorToSentry(error, stack));
    _navigateToInvariantError(error);
    return true;
  }

  void _navigateToInvariantError(Object error) {
    _navigatorKey.currentState?.pushNamedAndRemoveUntil(
      WalletRoutes.invariantErrorRoute,
      (route) => false,
      arguments: InvariantErrorScreenArgument(code: error.toString()).toJson(),
    );
  }
}

Future<void> _reportFatalDartErrorToSentry(Object error, StackTrace stack) async {
  if (!Sentry.isEnabled) return;

  await Sentry.captureEvent(
    createFatalDartErrorEvent(error),
    stackTrace: stack,
  );
}

@visibleForTesting
SentryEvent createFatalDartErrorEvent(Object error) {
  final mechanism = Mechanism(type: 'PlatformDispatcher.onError', handled: false);
  return SentryEvent(
    throwable: ThrowableMechanism(mechanism, error),
    level: SentryLevel.fatal,
  );
}
