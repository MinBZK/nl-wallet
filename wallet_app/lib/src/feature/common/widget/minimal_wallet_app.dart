import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter_native_splash/flutter_native_splash.dart';

import '../../../data/store/active_locale_provider.dart';
import '../../../data/store/impl/active_localization_delegate.dart';
import '../../../theme/wallet_theme.dart';

/// This is a very special variant of the wallet app which only provides the bare necessities.
/// It should ONLY be used in the special case where we want to render some UI above the normal [WalletApp] (i.e.
/// when the [WalletApp] is not yet part of the tree). A sample of this is the [RootChecker] which lives at the
/// top of the widget tree as a security precaution, so that we don't accidentally load anything into memory.
class MinimalWalletApp extends StatelessWidget {
  final Widget child;

  const MinimalWalletApp({super.key, required this.child});

  @override
  Widget build(BuildContext context) {
    // Needed since the [SplashScreen], which normally handles this might not be reached yet.
    FlutterNativeSplash.remove();
    final localizationDelegate = ActiveLocalizationDelegate();
    return RepositoryProvider<ActiveLocaleProvider>(
      create: (c) => localizationDelegate,
      child: MaterialApp(
        theme: WalletTheme.light,
        darkTheme: WalletTheme.dark,
        localizationsDelegates: [
          localizationDelegate,
          ...AppLocalizations.localizationsDelegates,
        ],
        home: child,
      ),
    );
  }
}
