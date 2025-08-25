import 'dart:io';

import 'package:fimber/fimber.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_driver/driver_extension.dart';
import 'package:flutter_native_splash/flutter_native_splash.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:sentry_flutter/sentry_flutter.dart';
import 'package:wallet_core/core.dart' as core;
import 'package:wallet_mock/mock.dart' as mock;

import 'environment.dart';
import 'src/di/wallet_dependency_provider.dart';
import 'src/feature/common/widget/flutter_app_configuration_provider.dart';
import 'src/feature/common/widget/privacy_cover.dart';
import 'src/feature/lock/auto_lock_observer.dart';
import 'src/feature/root/root_checker.dart';
import 'src/feature/update/update_checker.dart';
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

  if (Environment.mockRepositories) {
    core.WalletCore.initMock(api: mock.api);
  } else {
    final lib = Platform.isIOS || Platform.isMacOS ? ExternalLibrary.process(iKnowHowToUseIt: true) : null;
    await core.WalletCore.init(externalLibrary: lib);
  }

  await core.postInit();

  if (Environment.hasSentryDsn) {
    await SentryFlutter.init(
      (options) => options
        ..dsn = Environment.sentryDsn
        ..environment = Environment.sentryEnvironment
        ..release = Environment.sentryRelease() // default applies when SENTRY_RELEASE not set
        ..debug = kDebugMode
        ..beforeSend = beforeSend,
      appRunner: mainImpl,
    );
  } else {
    mainImpl();
  }
}

FutureOr<SentryEvent?> beforeSend(SentryEvent event, Hint hint) async {
  // Strip all breadcrumbs and exception values from the event
  event.breadcrumbs = null;
  event.exceptions?.forEach((ex) {
    ex.value = null;
  });
  return event;
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
            child: UpdateChecker(
              child: PrivacyCover(
                child: WalletApp(
                  navigatorKey: _navigatorKey,
                ),
              ),
            ),
          ),
        ),
      ),
    ),
  );
}
