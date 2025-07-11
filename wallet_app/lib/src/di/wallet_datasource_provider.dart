import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../data/source/impl/wallet_datasource_impl.dart';
import '../data/source/wallet_datasource.dart';
import '../data/store/active_locale_provider.dart';
import '../data/store/impl/active_localization_delegate.dart';
import '../data/store/impl/language_store_impl.dart';
import '../data/store/impl/tour_store_impl.dart';
import '../data/store/language_store.dart';
import '../data/store/tour_store.dart';
import '../wallet_core/typed/typed_wallet_core.dart';

class WalletDataSourceProvider extends StatelessWidget {
  final Widget child;
  final bool provideMocks;

  const WalletDataSourceProvider({required this.child, this.provideMocks = false, super.key});

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
        RepositoryProvider<TypedWalletCore>(create: (context) => TypedWalletCore(context.read())),
        RepositoryProvider<LanguageStore>(
          create: (context) => LanguageStoreImpl(SharedPreferences.getInstance),
        ),
        RepositoryProvider<TourStore>(
          create: (context) => TourStoreImpl(SharedPreferences.getInstance),
        ),
        RepositoryProvider<WalletDataSource>(
          create: (context) => WalletDataSourceImpl(context.read(), context.read()),
        ),
      ],
      child: child,
    );
  }
}
