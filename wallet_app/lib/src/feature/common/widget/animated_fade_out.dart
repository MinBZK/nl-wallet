import 'dart:async';

import 'package:after_layout/after_layout.dart';
import 'package:flutter/material.dart';

import '../../../wallet_constants.dart';

class AnimatedFadeOut extends StatefulWidget {
  final Widget child;

  const AnimatedFadeOut({required this.child, super.key});

  @override
  State<AnimatedFadeOut> createState() => _AnimatedFadeOutState();
}

class _AnimatedFadeOutState extends State<AnimatedFadeOut> with AfterLayoutMixin<AnimatedFadeOut> {
  double _opacity = 1;

  @override
  FutureOr<void> afterFirstLayout(BuildContext context) {
    setState(() => _opacity = 0.0);
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedOpacity(
      opacity: _opacity,
      duration: kDefaultAnimationDuration,
      child: widget.child,
    );
  }
}
