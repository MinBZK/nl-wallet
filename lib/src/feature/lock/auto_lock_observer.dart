import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:rxdart/rxdart.dart';

import '../../data/repository/wallet/wallet_repository.dart';
import '../../wallet_constants.dart';
import 'widget/interaction_detector.dart';

class AutoLockObserver extends StatefulWidget {
  final Widget child;

  const AutoLockObserver({required this.child, Key? key}) : super(key: key);

  @override
  State<AutoLockObserver> createState() => _AutoLockObserverState();
}

class _AutoLockObserverState extends State<AutoLockObserver> with WidgetsBindingObserver {
  final PublishSubject<void> _userInteractionStream = PublishSubject();
  final Stopwatch _backgroundStopwatch = Stopwatch();

  @override
  Widget build(BuildContext context) {
    return InteractionDetector(
      onInteraction: () => _resetIdleTimeout(),
      child: widget.child,
    );
  }

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    _userInteractionStream.debounceTime(kIdleLockTimeout).listen((event) => _lockWallet());
    if (WidgetsBinding.instance.lifecycleState != AppLifecycleState.resumed) {
      _lockWallet();
    } else {
      _resetIdleTimeout();
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
    if (_backgroundStopwatch.elapsed >= kBackgroundLockTimeout) _lockWallet();
    _backgroundStopwatch.reset();
  }

  void _lockWallet() {
    Fimber.d('Locking wallet!');
    context.read<WalletRepository>().lockWallet();
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }
}
