import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../data/source/memory/memory_wallet_datasource.dart';
import '../data/source/mock/mock_organization_datasource.dart';
import '../data/source/organization_datasource.dart';
import '../data/source/wallet_datasource.dart';
import '../data/store/impl/language_store_impl.dart';
import '../data/store/language_store.dart';
import '../rust_core.dart';

class WalletDataSourceProvider extends StatelessWidget {
  final Widget child;

  const WalletDataSourceProvider({required this.child, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MultiRepositoryProvider(
      providers: [
        RepositoryProvider<WalletDataSource>(
          create: (context) => MemoryWalletDataSource(),
        ),
        RepositoryProvider<OrganizationDataSource>(
          create: (context) => MockOrganizationDataSource(),
        ),
        RepositoryProvider<RustCore>(
          create: (context) => api,
        ),
        RepositoryProvider<LanguageStore>(
          create: (context) => LanguageStoreImpl(() => SharedPreferences.getInstance()),
        ),
      ],
      child: child,
    );
  }
}
