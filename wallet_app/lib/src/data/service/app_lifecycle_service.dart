import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:rxdart/rxdart.dart';

/// Service that provides the current [AppLifecycleState] to any component, without the need
/// of a [StatefulWidget]. This does require that the [AppLifecycleObserver] is part of the
/// widget tree.
class AppLifecycleService {
  /// The app is always in the foreground when it's initially started, so we default to the resumed state.
  final _appLifecycleStateSubject = BehaviorSubject<AppLifecycleState>.seeded(AppLifecycleState.resumed);

  AppLifecycleService();

  Stream<AppLifecycleState> observe() => _appLifecycleStateSubject.stream;

  void notifyStateChanged(AppLifecycleState state) => _appLifecycleStateSubject.add(state);
}

/// Widget that provides the [AppLifecycleState] to the [AppLifecycleService].
class AppLifecycleObserver extends StatefulWidget {
  final Widget? child;

  const AppLifecycleObserver({
    this.child,
    super.key,
  });

  @override
  State<AppLifecycleObserver> createState() => _AppLifecycleObserverState();
}

class _AppLifecycleObserverState extends State<AppLifecycleObserver> with WidgetsBindingObserver {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    context.read<AppLifecycleService>().notifyStateChanged(state);
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => widget.child ?? const SizedBox.shrink();
}
