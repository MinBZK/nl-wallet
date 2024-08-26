import 'dart:math';

import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import 'pin_dot.dart';

class PinField extends StatefulWidget {
  final int digits;
  final int enteredDigits;
  final PinFieldState state;

  /// The color used to draw the dots, defaults to [ColorScheme.onSurface]
  final Color? color;

  const PinField({
    required this.digits,
    required this.enteredDigits,
    required this.state,
    this.color,
    super.key,
  });

  @override
  State<PinField> createState() => _PinFieldState();
}

class _PinFieldState extends State<PinField> with TickerProviderStateMixin {
  late AnimationController _waveController;
  late AnimationController _amplitudeController;
  late AnimationController _shakeController;

  bool get isLoading => widget.state == PinFieldState.loading;

  @override
  void initState() {
    super.initState();
    // Setup animation controllers
    _waveController = AnimationController(vsync: this, duration: const Duration(milliseconds: 2000));
    _amplitudeController = AnimationController(vsync: this, duration: const Duration(milliseconds: 300));
    _shakeController = AnimationController(vsync: this, duration: const Duration(milliseconds: 500));

    // Repeat wave controller when state is loading
    _waveController.addStatusListener((status) {
      if (isLoading && status == AnimationStatus.completed) {
        _waveController.forward(from: 0);
      }
    });

    // Update UI when controllers fire
    _waveController.addListener(() => setState(() {}));
    _amplitudeController.addListener(() => setState(() {}));
    _shakeController.addListener(() => setState(() {}));
  }

  @override
  void didUpdateWidget(covariant PinField oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.state != widget.state) {
      switch (widget.state) {
        case PinFieldState.idle:
          _amplitudeController.reverse();
        case PinFieldState.error:
          _shakeController.forward(from: 0);
          _amplitudeController.reverse();
        case PinFieldState.loading:
          _waveController.forward(from: 0);
          _amplitudeController.forward();
      }
    }
  }

  @override
  void dispose() {
    _waveController.dispose();
    _amplitudeController.dispose();
    _shakeController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Semantics(
      attributedLabel:
          context.l10n.pinFieldAnnouncementLabel(widget.digits - widget.enteredDigits).toAttributedString(context),
      child: Transform.translate(
        offset: _calcShakeOffset(),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: List.generate(
            widget.digits,
            (index) => Transform.translate(
              offset: _calcWaveOffset(index),
              child: PinDot(
                checked: index < widget.enteredDigits,
                key: ValueKey('pinDot#$index'),
                color: widget.color ?? context.colorScheme.onSurface,
              ),
            ),
          ),
        ),
      ),
    );
  }

  Offset _calcWaveOffset(int index) {
    // Sets the intensity of the offset
    const amplitudeStrength = 8;
    // The actual amplitude used in the formula, this value is animated so we can always smoothly animate back to 0, regardless of when the animation ends.
    final amplitude = amplitudeStrength * _amplitudeController.value;
    // The input of our sine function, multiplied py pi to simplify reasoning over the sine.
    final x = _waveController.value * pi;
    // The period of the sinusoidal function, 2 means we go to 1 and -1 in one animation cycle
    const period = 2;
    // The horizontal translation, takes the index as an input to make the dots appear to be animating in order.
    final horizontalTranslation = (pi / widget.digits) * index;
    // Calculate the actual offset
    final dy = sin(period * x - horizontalTranslation);
    // We only care about the positive values, to avoid a 'snake' like animation.
    if (dy.isNegative) return Offset.zero;
    // The final result is multiplied by the amplitude to make the shift (more) noticeable and by -1 to animate 'up'.
    return Offset(0, dy * amplitude * -1);
  }

  Offset _calcShakeOffset() {
    const nrOfShakes = 3;
    const shakeOffset = 4;
    final sineValue = sin(nrOfShakes * 2 * pi * _shakeController.value);
    return Offset(sineValue * shakeOffset, 0);
  }
}

enum PinFieldState { idle, loading, error }
