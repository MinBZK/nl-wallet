import 'package:flutter/material.dart';

import '../../environment.dart';
import 'wallet_bloc_provider.dart';
import 'wallet_datasource_provider.dart';
import 'wallet_repository_provider.dart';
import 'wallet_service_provider.dart';
import 'wallet_usecase_provider.dart';

/// Widget that provides all the Dependencies, i.e.
/// Datasources, Repositories, Usecases, Services and Blocs
/// to the provided [child].
class WalletDependencyProvider extends StatelessWidget {
  final Widget child;
  final GlobalKey<NavigatorState> navigatorKey;

  const WalletDependencyProvider({
    required this.child,
    required this.navigatorKey,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return WalletDataSourceProvider(
      provideMocks: Environment.mockRepositories,
      child: WalletRepositoryProvider(
        provideMocks: Environment.mockRepositories,
        child: WalletUseCaseProvider(
          child: WalletServiceProvider(
            navigatorKey: navigatorKey,
            child: WalletBlocProvider(child: child),
          ),
        ),
      ),
    );
  }
}
