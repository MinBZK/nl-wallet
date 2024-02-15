import 'package:flutter/material.dart';

import '../../../wallet_constants.dart';

class AnimatedLinearProgressIndicator extends StatelessWidget {
  final double progress;

  const AnimatedLinearProgressIndicator({required this.progress, super.key});

  @override
  Widget build(BuildContext context) {
    return TweenAnimationBuilder<double>(
      builder: (context, progress, child) => LinearProgressIndicator(
        value: progress,
        borderRadius: BorderRadius.circular(8),
      ),
      duration: kDefaultAnimationDuration,
      tween: Tween<double>(end: progress),
    );
  }
}
