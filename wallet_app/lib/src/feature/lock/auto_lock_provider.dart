import 'package:flutter/widgets.dart';

class AutoLockProvider extends InheritedWidget {
  final VoidCallback resetIdleTimeout;

  final Function({required bool enabled}) setAutoLock;

  const AutoLockProvider({
    super.key,
    required this.resetIdleTimeout,
    required this.setAutoLock,
    required super.child,
  });

  static AutoLockProvider? of(BuildContext context) {
    return context.dependOnInheritedWidgetOfExactType<AutoLockProvider>();
  }

  @override
  bool updateShouldNotify(AutoLockProvider oldWidget) =>
      oldWidget.resetIdleTimeout != resetIdleTimeout || oldWidget.setAutoLock != setAutoLock;
}
