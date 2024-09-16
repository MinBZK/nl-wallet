import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter_native_splash/flutter_native_splash.dart';
import 'package:root_jailbreak_sniffer/rjsniffer.dart';

import '../../data/store/active_locale_provider.dart';
import '../../data/store/impl/active_localization_delegate.dart';
import '../../theme/wallet_theme.dart';
import 'root_detected_screen.dart';

/// Signature to replace the built-in root check (useful for testing). Return
/// true to indicate that the device *is* rooted / jailbroken.
typedef CheckForRoot = Future<bool> Function(BuildContext);

/// A widget that checks if the device is jailbroken (iOS) or rooted (Android).
/// If any of these states are detected it blocks any further access to the app
/// by providing a custom 'app is blocked' widget.
class RootChecker extends StatefulWidget {
  /// Option to replace the built-in root detection
  @visibleForTesting
  final CheckForRoot? customRootCheck;

  final Widget child;

  const RootChecker({
    required this.child,
    this.customRootCheck,
    super.key,
  });

  @override
  State<RootChecker> createState() => _RootCheckerState();
}

class _RootCheckerState extends State<RootChecker> with WidgetsBindingObserver {
  // Whether the device is Rooted or Jailbroken
  final ValueNotifier<bool> _hasRoot = ValueNotifier(false);

  @override
  void initState() {
    super.initState();
    _performRootCheck();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return ValueListenableBuilder(
      valueListenable: _hasRoot,
      builder: (c, value, child) {
        if (value) {
          // Needed since the [SplashScreen], which normally handles this might not be reached yet.
          FlutterNativeSplash.remove();
          // We don't have anything provided at this stage, make sure this is tested and using basic Widgets.
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
              home: const RootDetectedScreen(),
            ),
          );
        }
        return child!;
      },
      child: widget.child,
    );
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) _performRootCheck();
  }

  Future<void> _performRootCheck() async {
    if (widget.customRootCheck != null) {
      _hasRoot.value = await widget.customRootCheck!(context);
    } else {
      _hasRoot.value = await Rjsniffer.amICompromised() ?? false;
    }
  }
}
