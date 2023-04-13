import 'package:flutter/material.dart';

import '../../../wallet_constants.dart';

class AnimatedLinearProgressIndicator extends StatelessWidget {
  final double progress;

  const AnimatedLinearProgressIndicator({required this.progress, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return TweenAnimationBuilder<double>(
      builder: (context, progress, child) => LinearProgressIndicator(value: progress),
      duration: kDefaultAnimationDuration,
      tween: Tween<double>(end: progress),
    );
  }
}
