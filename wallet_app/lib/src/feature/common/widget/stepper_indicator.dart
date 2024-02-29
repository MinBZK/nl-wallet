import 'package:flutter/material.dart';

import 'animated_linear_progress_indicator.dart';

class StepperIndicator extends StatelessWidget {
  final double progress;

  const StepperIndicator({required this.progress, super.key});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: AnimatedLinearProgressIndicator(progress: progress),
    );
  }
}
