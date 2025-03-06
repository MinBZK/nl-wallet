import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import '../l10n/generated/app_localizations.dart';
import 'data/store/impl/active_localization_delegate.dart';
import 'localization/preferred_locale_cubit.dart';
import 'navigation/wallet_routes.dart';
import 'theme/wallet_theme.dart';
import 'util/extension/build_context_extension.dart';

class WalletApp extends StatelessWidget {
  final GlobalKey<NavigatorState> navigatorKey;

  const WalletApp({required this.navigatorKey, super.key});

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<PreferredLocaleCubit, Locale?>(
      builder: (context, locale) {
        return MaterialApp(
          localizationsDelegates: [
            ...AppLocalizations.localizationsDelegates,
            context.read<ActiveLocalizationDelegate>(),
          ],
          supportedLocales: AppLocalizations.supportedLocales,
          navigatorKey: navigatorKey,
          locale: locale,
          onGenerateTitle: (BuildContext context) => context.l10n.appTitle,
          theme: WalletTheme.light,
          darkTheme: WalletTheme.dark,
          onGenerateInitialRoutes: WalletRoutes.initialRoutes,
          onGenerateRoute: WalletRoutes.routeFactory,
        );
      },
    );
  }
}
