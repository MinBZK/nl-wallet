import 'package:flutter/material.dart';

import 'wallet_bloc_provider.dart';
import 'wallet_datasource_provider.dart';
import 'wallet_mapper_provider.dart';
import 'wallet_repository_provider.dart';
import 'wallet_service_provider.dart';
import 'wallet_usecase_provider.dart';

/// Widget that provides all the Dependencies, i.e.
/// DataSources, Repositories, UseCases, Services and BLoCs
/// to the provided [builder].
class WalletDependencyProvider extends StatelessWidget {
  final WidgetBuilder builder;
  final GlobalKey<NavigatorState> navigatorKey;

  const WalletDependencyProvider({
    required this.builder,
    required this.navigatorKey,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return WalletMapperProvider(
      child: WalletDataSourceProvider(
        child: WalletRepositoryProvider(
          child: WalletUseCaseProvider(
            child: WalletServiceProvider(
              navigatorKey: navigatorKey,
              child: WalletBlocProvider(
                child: Builder(
                  builder: builder,
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
