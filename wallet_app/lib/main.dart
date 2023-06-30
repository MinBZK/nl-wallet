import 'package:fimber/fimber.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_driver/driver_extension.dart';
import 'src/feature/common/widget/privacy_cover.dart';

import 'src/di/wallet_dependency_provider.dart';
import 'src/feature/common/widget/flutter_app_configuration_provider.dart';
import 'src/feature/lock/auto_lock_observer.dart';
import 'src/wallet_app.dart';
import 'src/wallet_app_bloc_observer.dart';
import 'src/wallet_error_handler.dart';

final GlobalKey<NavigatorState> _navigatorKey = GlobalKey<NavigatorState>();

void main() async {
  // Propagate uncaught errors
  final errorHandler = WalletErrorHandler();
  PlatformDispatcher.instance.onError = (error, stack) => errorHandler.handlerError(error, stack);

  // Appium specific setup
  if (kProfileMode || kDebugMode) {
    enableFlutterDriverExtension();
  }

  WidgetsFlutterBinding.ensureInitialized();

  // Debug specific setup
  if (kDebugMode) {
    Fimber.plantTree(DebugTree());
    Bloc.observer = WalletAppBlocObserver();
  }

  runApp(
    WalletDependencyProvider(
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
  );
}
