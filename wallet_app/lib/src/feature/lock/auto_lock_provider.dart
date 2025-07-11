import 'package:flutter/widgets.dart';

class AutoLockProvider extends InheritedWidget {
  final VoidCallback resetIdleTimeout;

  const AutoLockProvider({
    super.key,
    required this.resetIdleTimeout,
    required super.child,
  });

  static AutoLockProvider? of(BuildContext context) {
    return context.dependOnInheritedWidgetOfExactType<AutoLockProvider>();
  }

  @override
  bool updateShouldNotify(AutoLockProvider oldWidget) {
    return oldWidget.resetIdleTimeout != resetIdleTimeout;
  }
}
