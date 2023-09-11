import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

/// Service that provides the current [AppLifecycleState] to any component, without the need
/// of a [StatefulWidget]. This does require that the [AppLifecycleObserver] is part of the
/// widget tree.
class AppLifecycleService {
  final _stateController = StreamController<AppLifecycleState>.broadcast();

  AppLifecycleService();

  Stream<AppLifecycleState> observe() => _stateController.stream;

  void notifyStateChanged(AppLifecycleState state) => _stateController.add(state);
}

/// Widget that provides the [AppLifecycleState] to the [AppLifecycleService].
class AppLifecycleObserver extends StatefulWidget {
  final Widget? child;

  const AppLifecycleObserver({
    this.child,
    Key? key,
  }) : super(key: key);

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
