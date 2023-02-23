import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import 'localization/preferred_locale_cubit.dart';
import 'theme/wallet_theme.dart';
import 'wallet_routes.dart';

class WalletApp extends StatelessWidget {
  final GlobalKey<NavigatorState> navigatorKey;

  const WalletApp({required this.navigatorKey, super.key});

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<PreferredLocaleCubit, Locale?>(
      builder: (context, locale) {
        return MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          navigatorKey: navigatorKey,
          locale: locale,
          onGenerateTitle: (BuildContext context) => AppLocalizations.of(context).appTitle,
          theme: WalletTheme.light,
          darkTheme: WalletTheme.dark,
          themeMode: ThemeMode.light,
          onGenerateInitialRoutes: WalletRoutes.initialRoutes,
          onGenerateRoute: WalletRoutes.routeFactory,
        );
      },
    );
  }
}
