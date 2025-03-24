import 'dart:async';
import 'dart:ui';

import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:rxdart/rxdart.dart';

import '../../data/repository/wallet/wallet_repository.dart';
import '../../domain/model/configuration/flutter_app_configuration.dart';
import 'widget/interaction_detector.dart';

class AutoLockObserver extends StatefulWidget {
  final Widget child;
  final FlutterAppConfiguration configuration;

  const AutoLockObserver({
    required this.child,
    required this.configuration,
    super.key,
  });

  @override
  State<AutoLockObserver> createState() => _AutoLockObserverState();
}

class _AutoLockObserverState extends State<AutoLockObserver> with WidgetsBindingObserver {
  final PublishSubject<void> _userInteractionStream = PublishSubject();
  final Stopwatch _backgroundStopwatch = Stopwatch();
  StreamSubscription? _inactiveSubscription;

  @override
  Widget build(BuildContext context) {
    return InteractionDetector(
      onInteraction: _resetIdleTimeout,
      child: widget.child,
    );
  }

  @override
  void initState() {
    super.initState();

    _setupNoInteractionListener();
    _setupSemanticActionListener();

    WidgetsBinding.instance.addObserver(this);
    if (WidgetsBinding.instance.lifecycleState != AppLifecycleState.resumed) {
      _lockWallet();
    } else {
      _resetIdleTimeout();
    }
  }

  void _setupSemanticActionListener() {
    PlatformDispatcher.instance.onSemanticsActionEvent = (SemanticsActionEvent event) {
      if (event.type != SemanticsAction.didLoseAccessibilityFocus) _resetIdleTimeout();
      try {
        WidgetsBinding.instance.performSemanticsAction(event);
      } catch (ex) {
        Fimber.e('Failed to propagate semantics action: $event', ex: ex);
      }
    };
  }

  void _setupNoInteractionListener() {
    _inactiveSubscription?.cancel();
    _inactiveSubscription =
        _userInteractionStream.debounceTime(widget.configuration.idleLockTimeout).listen((event) => _lockWallet());
  }

  @override
  void didUpdateWidget(AutoLockObserver oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.configuration.idleLockTimeout != widget.configuration.idleLockTimeout) {
      _setupNoInteractionListener();
    }
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    Fimber.d('AppLifecycleState: ${state.name}');
    if (state == AppLifecycleState.resumed) _checkAndResetStopwatch();
    if (state == AppLifecycleState.inactive) _startStopwatch();
    if (state == AppLifecycleState.paused) _startStopwatch();
    if (state == AppLifecycleState.detached) _startStopwatch();
  }

  void _resetIdleTimeout() => _userInteractionStream.add(null);

  /// Starts the background lock stopwatch. If it's already running the call is ignored.
  void _startStopwatch() => _backgroundStopwatch.start();

  /// Locks the app if needed, and reset the stopwatch for future use.
  void _checkAndResetStopwatch() {
    _backgroundStopwatch.stop();
    if (_backgroundStopwatch.elapsed >= widget.configuration.backgroundLockTimeout) _lockWallet();
    _backgroundStopwatch.reset();
  }

  void _lockWallet() {
    Fimber.d('Locking wallet!');
    context.read<WalletRepository>().lockWallet();
  }

  @override
  void dispose() {
    _inactiveSubscription?.cancel();
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }
}
