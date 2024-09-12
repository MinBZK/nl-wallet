import 'package:fimber/fimber.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_driver/driver_extension.dart';
import 'package:flutter_native_splash/flutter_native_splash.dart';
import 'package:sentry_flutter/sentry_flutter.dart';

import 'environment.dart';
import 'src/di/wallet_dependency_provider.dart';
import 'src/feature/common/widget/flutter_app_configuration_provider.dart';
import 'src/feature/common/widget/privacy_cover.dart';
import 'src/feature/lock/auto_lock_observer.dart';
import 'src/feature/root/root_checker.dart';
import 'src/wallet_app.dart';
import 'src/wallet_app_bloc_observer.dart';
import 'src/wallet_error_handler.dart';

final GlobalKey<NavigatorState> _navigatorKey = GlobalKey<NavigatorState>();

void main() async {
  // Appium specific setup
  if (kProfileMode || kDebugMode) {
    enableFlutterDriverExtension();
  }

  final widgetsBinding = WidgetsFlutterBinding.ensureInitialized();
  FlutterNativeSplash.preserve(widgetsBinding: widgetsBinding);

  // Propagate uncaught errors
  final errorHandler = WalletErrorHandler();
  PlatformDispatcher.instance.onError = errorHandler.handleError;

  if (Environment.hasSentryDsn) {
    await SentryFlutter.init(
      (options) => options
        ..dsn = Environment.sentryDsn
        ..environment = Environment.sentryEnvironment
        ..release = Environment.sentryRelease() // default applies when SENTRY_RELEASE not set
        ..debug = kDebugMode,
      appRunner: mainImpl,
    );
  } else {
    mainImpl();
  }
}

//ignore: avoid_void_async
void mainImpl() async {
  // Debug specific setup
  if (kDebugMode) {
    Fimber.plantTree(DebugTree());
    Bloc.observer = WalletAppBlocObserver();
  }

  runApp(
    RootChecker(
      child: WalletDependencyProvider(
        navigatorKey: _navigatorKey,
        child: FlutterAppConfigurationProvider(
          builder: (config) => AutoLockObserver(
            configuration: config,
            child: PrivacyCover(
              child: WalletApp(
                navigatorKey: _navigatorKey,
              ),
            ),
          ),
        ),
      ),
    ),
  );
}
