import 'dart:async';

import 'package:after_layout/after_layout.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../util/manager/biometric_unlock_manager.dart';

/// Briefly delay the biometrics trigger, allowing the UI to settle
/// and the [BiometricUnlockManager] to finish processing potential events.
const kTriggerDelay = Duration(milliseconds: 50);

/// Fires the [onTriggerBiometricUnlock] to notify the parent widget that a
/// biometric unlock request is desired. The decision to fire the callback
/// is made in cooperation with the [BiometricUnlockManager].
class AutoBiometricUnlockTrigger extends StatefulWidget {
  final Function(BuildContext) onTriggerBiometricUnlock;
  final Widget? child;

  const AutoBiometricUnlockTrigger({
    this.child,
    required this.onTriggerBiometricUnlock,
    super.key,
  });

  @override
  State<AutoBiometricUnlockTrigger> createState() => _AutoBiometricUnlockTriggerState();
}

class _AutoBiometricUnlockTriggerState extends State<AutoBiometricUnlockTrigger>
    with WidgetsBindingObserver, AfterLayoutMixin {
  late BiometricUnlockManager _biometricUnlockManager;

  @override
  void initState() {
    super.initState();
    _biometricUnlockManager = context.read();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  Future<void> didChangeAppLifecycleState(AppLifecycleState state) async {
    if (state == AppLifecycleState.resumed) {
      await Future.delayed(kTriggerDelay);
      if (_biometricUnlockManager.getAndSetShouldTriggerUnlock(updatedValue: false)) {
        final context = this.context;
        if (context.mounted) widget.onTriggerBiometricUnlock(context);
      }
    }
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => widget.child ?? const SizedBox.shrink();

  @override
  FutureOr<void> afterFirstLayout(BuildContext context) async {
    if (_biometricUnlockManager.getAndSetShouldTriggerUnlock(updatedValue: false)) {
      if (context.mounted) widget.onTriggerBiometricUnlock(context);
    }
  }
}
