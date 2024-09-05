import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import 'animated_linear_progress_indicator.dart';

class StepperIndicator extends StatelessWidget {
  final int currentStep, totalSteps;

  double get progress => currentStep / totalSteps;

  const StepperIndicator({
    this.currentStep = 1,
    this.totalSteps = 5,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Semantics(
      attributedLabel: context.l10n.generalWCAGStepper(currentStep, totalSteps).toAttributedString(context),
      excludeSemantics: true,
      child: Hero(
        tag: 'stepper_indicator',
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 0.5),
          child: AnimatedLinearProgressIndicator(progress: progress),
        ),
      ),
    );
  }
}
