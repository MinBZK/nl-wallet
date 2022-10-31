import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter_localizations/flutter_localizations.dart';

import 'theme/wallet_theme.dart';
import 'wallet_routes.dart';

class WalletApp extends StatelessWidget {
  const WalletApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      restorationScopeId: 'app',
      localizationsDelegates: const [
        AppLocalizations.delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
      ],
      supportedLocales: const [
        Locale('en', ''), // English, no country code
        Locale('nl', ''), // Dutch, no country code
      ],
      onGenerateTitle: (BuildContext context) => AppLocalizations.of(context).appTitle,
      theme: WalletTheme.light,
      onGenerateInitialRoutes: WalletRoutes.initialRoutes,
      onGenerateRoute: WalletRoutes.routeFactory,
    );
  }
}
