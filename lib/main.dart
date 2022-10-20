import 'package:fimber/fimber.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import 'src/di/wallet_repository_provider.dart';
import 'src/di/wallet_usecase_provider.dart';
import 'src/wallet_app.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  if (kDebugMode) Fimber.plantTree(DebugTree());
  runApp(
    const WalletRepositoryProvider(
      child: WalletUseCaseProvider(
        child: WalletApp(),
      ),
    ),
  );
}
