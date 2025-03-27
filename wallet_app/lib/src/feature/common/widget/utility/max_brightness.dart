import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:screen_brightness/screen_brightness.dart';

/// When this Widget is part of the current WidgetTree the screen will be boosted to max brightness
class MaxBrightness extends StatefulWidget {
  final Widget child;

  const MaxBrightness({
    required this.child,
    super.key,
  });

  @override
  State<MaxBrightness> createState() => _MaxBrightnessState();
}

class _MaxBrightnessState extends State<MaxBrightness> {
  final _screenBrightness = ScreenBrightness();

  @override
  void initState() {
    super.initState();
    setMaxBrightness();
  }

  @override
  Widget build(BuildContext context) => widget.child;

  @override
  void dispose() {
    resetBrightness();
    super.dispose();
  }

  Future<void> setMaxBrightness() async {
    try {
      await _screenBrightness.setApplicationScreenBrightness(1);
    } catch (e, stack) {
      Fimber.e('Failed to set max brightness', ex: e, stacktrace: stack);
    }
  }

  Future<void> resetBrightness() async {
    try {
      await _screenBrightness.resetApplicationScreenBrightness();
    } catch (e, stack) {
      Fimber.e('Failed to set reset brightness', ex: e, stacktrace: stack);
    }
  }
}
