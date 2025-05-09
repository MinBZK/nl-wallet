import 'dart:async';
import 'dart:ui';

import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:rxdart/rxdart.dart';

import '../../data/service/navigation_service.dart';
import '../../domain/model/configuration/flutter_app_configuration.dart';
import '../../domain/usecase/wallet/is_wallet_registered_and_unlocked_usecase.dart';
import '../../domain/usecase/wallet/lock_wallet_usecase.dart';
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
  StreamSubscription? _inactiveWarningSubscription;
  StreamSubscription? _inactiveSubscription;

  /// When true, semantic events will reset the 'user idle' ([_userInteractionStream]) timer
  bool _semanticsTimerResetEnabled = true;

  LockWalletUseCase get _lockWalletUseCase => context.read();

  IsWalletRegisteredAndUnlockedUseCase get _isWalletRegisteredAndUnlockedUseCase => context.read();

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
      _lockWallet(null);
    } else {
      _resetIdleTimeout();
    }
  }

  void _setupSemanticActionListener() {
    PlatformDispatcher.instance.onSemanticsActionEvent = (SemanticsActionEvent action) {
      if (_semanticsTimerResetEnabled && action.type != SemanticsAction.didLoseAccessibilityFocus) _resetIdleTimeout();
      try {
        final Object? arguments = action.arguments;
        // Decode the [SemanticsActionEvent] before passing it on. Needed to avoid ex. & support scroll like events.
        final SemanticsActionEvent decodedAction = arguments is ByteData
            ? action.copyWith(arguments: const StandardMessageCodec().decodeMessage(arguments))
            : action;
        WidgetsBinding.instance.performSemanticsAction(decodedAction);
      } catch (ex) {
        Fimber.e('Failed to propagate semantics action: $action', ex: ex);
      }
    };
  }

  void _setupNoInteractionListener() {
    // Idle warning dialog timeout
    _inactiveWarningSubscription?.cancel();
    _inactiveWarningSubscription = _userInteractionStream
        .debounceTime(widget.configuration.idleWarningTimeout)
        .asyncMap((_) async => _isWalletRegisteredAndUnlockedUseCase.invoke())
        .where((showWarning) => showWarning)
        .listen(_showIdleDialog);
    // Idle lock timeout
    _inactiveSubscription?.cancel();
    _inactiveSubscription = _userInteractionStream
        .debounceTime(widget.configuration.idleLockTimeout)
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

  void _resetIdleTimeout() => _userInteractionStream.add(null);

  /// Starts the background lock stopwatch. If it's already running the call is ignored.
  void _startStopwatch() => _backgroundStopwatch.start();

  /// Locks the app if needed, and reset the stopwatch for future use.
  void _checkAndResetStopwatch() {
    _backgroundStopwatch.stop();
    if (_backgroundStopwatch.elapsed >= widget.configuration.backgroundLockTimeout) _lockWallet(null);
    _backgroundStopwatch.reset();
  }

  // Show the Timeout Warning Dialog (not called directly due to missing theme for the local context)
  void _showIdleDialog(_) {
    try {
      // Briefly disable the semantics idle reset timer to avoid resetting the timer when the dialog grabs focus
      _semanticsTimerResetEnabled = false;
      context.read<NavigationService>().showDialog(WalletDialogType.idleWarning, dismissOpenDialogs: true);
    } finally {
      // Re-enable the idle timer
      Future.delayed(Duration(seconds: 1)).then((_) => _semanticsTimerResetEnabled = true);
    }
  }

  void _lockWallet(_) {
    Fimber.d('Locking wallet!');
    _lockWalletUseCase.invoke();
  }

  @override
  void dispose() {
    _inactiveWarningSubscription?.cancel();
    _inactiveSubscription?.cancel();
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }
}
