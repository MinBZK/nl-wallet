import 'package:fimber/fimber.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'src/di/wallet_repository_provider.dart';
import 'src/di/wallet_usecase_provider.dart';
import 'src/feature/lock/auto_lock_observer.dart';
import 'src/wallet_app.dart';
import 'src/wallet_app_bloc_observer.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  if (kDebugMode) {
    Fimber.plantTree(DebugTree());
    Bloc.observer = WalletAppBlocObserver();
  }
  runApp(
    const WalletRepositoryProvider(
      child: WalletUseCaseProvider(
        child: AutoLockObserver(
          child: WalletApp(),
        ),
      ),
    ),
  );
}
