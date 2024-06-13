import 'dart:async';

import 'package:after_layout/after_layout.dart';
import 'package:flutter/material.dart';

import '../../../wallet_constants.dart';

class AnimatedFadeIn extends StatefulWidget {
  final Widget child;

  const AnimatedFadeIn({required this.child, super.key});

  @override
  State<AnimatedFadeIn> createState() => _AnimatedFadeInState();
}

class _AnimatedFadeInState extends State<AnimatedFadeIn> with AfterLayoutMixin<AnimatedFadeIn> {
  double _opacity = 0;

  @override
  FutureOr<void> afterFirstLayout(BuildContext context) {
    setState(() => _opacity = 1.0);
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
