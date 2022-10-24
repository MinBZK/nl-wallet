import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../data/repository/wallet/mock_wallet_repository.dart';
import '../data/repository/wallet/wallet_repository.dart';

/// This widget is responsible for initializing and providing all usecases.
/// Most likely to be used once at the top (app) level, but notable below the
/// [WalletRepositoryProvider] as usecases will likely depend on one or more
/// Repositories.
class WalletRepositoryProvider extends StatelessWidget {
  final Widget child;

  const WalletRepositoryProvider({required this.child, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MultiRepositoryProvider(
      providers: [
        RepositoryProvider<WalletRepository>(
          create: (context) => MockWalletRepository(),
        )
      ],
      child: child,
    );
  }
}
