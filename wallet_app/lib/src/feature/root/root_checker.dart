import 'dart:async';

import 'package:flutter/material.dart';
import 'package:root_jailbreak_sniffer/rjsniffer.dart';

import '../common/widget/minimal_wallet_app.dart';
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
      builder: (c, hasRoot, child) {
        if (hasRoot) return const MinimalWalletApp(child: RootDetectedScreen());
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
