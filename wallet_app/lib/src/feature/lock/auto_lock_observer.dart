import 'dart:async';
import 'dart:ui';

import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:rxdart/rxdart.dart';

import '../../../environment.dart';
import '../../data/service/navigation_service.dart';
import '../../data/service/semantics_event_service.dart';
import '../../domain/model/configuration/flutter_app_configuration.dart';
import '../../domain/usecase/wallet/is_wallet_registered_and_unlocked_usecase.dart';
import '../../domain/usecase/wallet/lock_wallet_usecase.dart';
import 'auto_lock_provider.dart';
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
  StreamSubscription? _semanticsSubscription;
  StreamSubscription? _inactiveWarningSubscription;
  StreamSubscription? _inactiveSubscription;

  /// Whether auto-locking due to inactivity or backgrounding is currently active.
  ///
  /// This can be toggled at runtime by calling [AutoLockProvider.of(context).enableAutoLock(bool)] to temporarily
  /// prevent the wallet from locking.
  ///
  /// Defaults to `true`, but can be globally disabled for development or
  /// debugging purposes by setting [Environment.disableAutoLock] to `true`.
  bool _autoLockEnabled = !Environment.disableAutoLock;

  /// When true, semantic events will reset the 'user idle' ([_userInteractionStream]) timer
  bool _semanticsTimerResetEnabled = true;

  /// Use case that checks if a wallet is registered and currently unlocked.
  IsWalletRegisteredAndUnlockedUseCase get _isWalletRegisteredAndUnlockedUseCase => context.read();

  @override
  Widget build(BuildContext context) {
    return AutoLockProvider(
      resetIdleTimeout: _resetIdleTimeout,
      setAutoLock: _setAutoLock,
      child: InteractionDetector(
        onInteraction: _resetIdleTimeout,
        child: widget.child,
      ),
    );
  }

  @override
  void initState() {
    super.initState();

    _setupNoInteractionListener();
    _setupSemanticActionListener();

    WidgetsBinding.instance.addObserver(this);
    if (WidgetsBinding.instance.lifecycleState != AppLifecycleState.resumed) {
      _lockWallet(null);
    } else {
      _resetIdleTimeout();
    }
  }

  void _setupSemanticActionListener() {
    _semanticsSubscription = context.read<SemanticsEventService>().actionEventStream.listen((action) {
      if (_semanticsTimerResetEnabled && action.type != SemanticsAction.didLoseAccessibilityFocus) _resetIdleTimeout();
    });
  }

  void _setupNoInteractionListener() {
    // Idle warning dialog timeout
    _inactiveWarningSubscription?.cancel();
    _inactiveWarningSubscription = _userInteractionStream
        .debounceTime(widget.configuration.idleWarningTimeout)
        .where((_) => _autoLockEnabled)
        .asyncMap((_) async => _isWalletRegisteredAndUnlockedUseCase.invoke())
        .where((showWarning) => showWarning)
        .listen(_showIdleDialog);
    // Idle lock timeout
    _inactiveSubscription?.cancel();
    _inactiveSubscription = _userInteractionStream
        .debounceTime(widget.configuration.idleLockTimeout)
        .where((_) => _autoLockEnabled)
        .asyncMap((_) async => _isWalletRegisteredAndUnlockedUseCase.invoke())
        .where((shouldLock) => shouldLock)
        .listen(_lockWallet);
  }

  @override
  void didUpdateWidget(AutoLockObserver oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.configuration != widget.configuration) {
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

  void _setAutoLock({required bool enabled}) {
    if (_autoLockEnabled == enabled) return; // Ignore, nothing changed
    if (Environment.disableAutoLock) return; // Ignore, auto lock is globally disabled
    Fimber.d('Auto lock enabled: $enabled');
    _autoLockEnabled = enabled;
    if (_autoLockEnabled) _resetIdleTimeout(); // Reset idle timeout (previous idle event might have been muted)
  }

  void _resetIdleTimeout() => _userInteractionStream.add(null);

  /// Starts the background lock stopwatch. If it's already running the call is ignored.
  void _startStopwatch() => _backgroundStopwatch.start();

  /// Locks the app if needed, and reset the stopwatch for future use.
  void _checkAndResetStopwatch() {
    _backgroundStopwatch.stop();
    if (_autoLockEnabled && _backgroundStopwatch.elapsed >= widget.configuration.backgroundLockTimeout) {
      _lockWallet(null);
    }
    _backgroundStopwatch.reset();
  }

  // Show the Timeout Warning Dialog (not called directly due to missing theme for the local context)
  void _showIdleDialog(bool _) {
    try {
      // Briefly disable the semantics idle reset timer to avoid resetting the timer when the dialog grabs focus
      _semanticsTimerResetEnabled = false;
      context.read<NavigationService>().showDialog(WalletDialogType.idleWarning, dismissOpenDialogs: true);
    } finally {
      // Re-enable the idle timer
      Future.delayed(const Duration(seconds: 1)).then((_) => _semanticsTimerResetEnabled = true);
    }
  }

  void _lockWallet(dynamic _) {
    Fimber.d('Locking wallet!');
    context.read<LockWalletUseCase>().invoke();
  }

  @override
  void dispose() {
    _semanticsSubscription?.cancel();
    _inactiveWarningSubscription?.cancel();
    _inactiveSubscription?.cancel();
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }
}
