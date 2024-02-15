import 'dart:async';

import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:sensors_plus/sensors_plus.dart';

const _kGyroMultiplier = 10.0;

/// Draws the provided [gradient] on top of the [child] and translates it based on the device's gyroscope events.
class GyroGradientOverlay extends StatefulWidget {
  final Gradient gradient;
  final Widget child;

  const GyroGradientOverlay({
    required this.gradient,
    required this.child,
    super.key,
  });

  @override
  State<GyroGradientOverlay> createState() => _GyroGradientOverlayState();
}

class _GyroGradientOverlayState extends State<GyroGradientOverlay> {
  StreamSubscription? _gyroscopeSubscription;
  GyroscopeEvent? _gyroscopeEvent;

  @override
  void initState() {
    super.initState();
    _gyroscopeSubscription = gyroscopeEventStream(samplingPeriod: SensorInterval.uiInterval).listen(
      (event) => setState(() => _gyroscopeEvent = event),
      onError: (error) => Fimber.e('Could not read gyroscope events', ex: error),
      cancelOnError: true,
    );
  }

  @override
  void dispose() {
    _gyroscopeSubscription?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return ShaderMask(
      shaderCallback: (Rect bounds) => widget.gradient.createShader(
        bounds.translate(
          (_gyroscopeEvent?.x ?? 0) * _kGyroMultiplier,
          (_gyroscopeEvent?.y ?? 0) * _kGyroMultiplier,
        ),
      ),
      child: widget.child,
    );
  }
}
