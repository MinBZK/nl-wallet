import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wallet_core/core.dart' as core;
import 'package:wallet_mock/mock.dart' as mock;

import '../data/source/impl/core_wallet_datasource.dart';
import '../data/source/mock/mock_organization_datasource.dart';
import '../data/source/mock/mock_wallet_datasource.dart';
import '../data/source/organization_datasource.dart';
import '../data/source/wallet_datasource.dart';
import '../data/store/active_locale_provider.dart';
import '../data/store/impl/active_localization_delegate.dart';
import '../data/store/impl/language_store_impl.dart';
import '../data/store/language_store.dart';
import '../wallet_core/typed/typed_wallet_core.dart';

class WalletDataSourceProvider extends StatelessWidget {
  final Widget child;
  final bool provideMocks;

  const WalletDataSourceProvider({required this.child, this.provideMocks = false, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MultiRepositoryProvider(
      providers: [
        RepositoryProvider<ActiveLocalizationDelegate>(
          create: (context) => ActiveLocalizationDelegate(),
        ),
        RepositoryProvider<ActiveLocaleProvider>(
          /// Re-exposing the [ActiveLocalizationDelegate] as [ActiveLocaleProvider] for saner lookup.
          create: (context) => context.read<ActiveLocalizationDelegate>(),
        ),
        RepositoryProvider<OrganizationDataSource>(
          create: (context) => MockOrganizationDataSource(),
        ),
        RepositoryProvider<TypedWalletCore>(
          create: (context) => TypedWalletCore(provideMocks ? mock.api : core.api, context.read()),
        ),
        RepositoryProvider<LanguageStore>(
          create: (context) => LanguageStoreImpl(() => SharedPreferences.getInstance()),
        ),
        RepositoryProvider<WalletDataSource>(
          create: (context) => provideMocks
              ? MockWalletDataSource()
              : CoreWalletDataSource(context.read(), context.read(), context.read()),
        ),
      ],
      child: child,
    );
  }
}
