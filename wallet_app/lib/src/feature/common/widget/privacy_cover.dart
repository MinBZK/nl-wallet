import 'dart:io';

import 'package:flutter/material.dart';

import '../../../theme/dark_wallet_theme.dart';
import '../../../theme/light_wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import 'wallet_logo.dart';

/// Covers all content of the app when the app enters the background. This only works on iOS.
/// Android privacy is handled by setting FLAG_SECURE in [MainActivity.kt]
class PrivacyCover extends StatefulWidget {
  final Widget child;

  const PrivacyCover({required this.child, super.key});

  @override
  State<PrivacyCover> createState() => _PrivacyCoverState();
}

class _PrivacyCoverState extends State<PrivacyCover> with WidgetsBindingObserver {
  bool get hideContent => WidgetsBinding.instance.lifecycleState != AppLifecycleState.resumed;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    // This solution doesn't work on Android, simply returning child as a tiny performance optimisation.
    if (Platform.isAndroid) return widget.child;

    return Stack(
      alignment: Alignment.center,
      children: [
        widget.child,
        if (hideContent)
          Container(
            color: _resolveBgColor(context),
            alignment: Alignment.center,
            child: const WalletLogo(size: 100),
          ),
      ],
    );
  }

  /// Get the background color based on the device settings (i.e. darkMode)
  /// We can't simply use the [Theme] since this Widget can live above the
  /// [MaterialApp], which provides the [Theme] to all of it's children.
  Color _resolveBgColor(BuildContext context) {
    return context.isDeviceInDarkMode ? DarkWalletTheme.colorScheme.surface : LightWalletTheme.colorScheme.surface;
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    setState(() {});
  }
}
