import 'dart:math';

import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';

const double _selectedStepHeight = 6;
const double _selectedStepWidth = 16;
const double _stepHeight = 4;
const double _stepWidth = 4;

class IntroductionProgressStepper extends StatelessWidget {
  final double currentStep;
  final int totalSteps;

  const IntroductionProgressStepper({
    required this.currentStep,
    required this.totalSteps,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final List<Widget> steps = List.generate(
      totalSteps,
      (index) {
        if (currentStep.floor() == index) {
          final progress = currentStep - currentStep.floor();
          return _buildCurrentStep(context, 1 - progress);
        } else if (currentStep.ceil() == index) {
          final progress = currentStep.ceil() - currentStep;
          return _buildCurrentStep(context, 1 - progress);
        } else {
          return _buildStep(context);
        }
      },
    );

    final currentSemanticsStep = currentStep.toInt() + 1;
    return Semantics(
      attributedLabel:
          context.l10n.pageIndicatorSemanticsLabel(currentSemanticsStep, totalSteps).toAttributedString(context),
      currentValueLength: currentSemanticsStep,
      maxValueLength: totalSteps,
      child: Wrap(
        crossAxisAlignment: WrapCrossAlignment.center,
        spacing: 8,
        children: steps,
      ),
    );
  }

  Widget _buildStep(BuildContext context) {
    return SizedBox(
      height: _selectedStepHeight, //Makes sure all bubbles are always center aligned
      child: SizedBox(
        width: _stepWidth,
        height: _stepHeight,
        child: DecoratedBox(
          decoration: BoxDecoration(
            color: context.theme.primaryColorDark,
            shape: BoxShape.circle,
          ),
        ),
      ),
    );
  }

  Widget _buildCurrentStep(BuildContext context, double size) {
    return SizedBox(
      width: max(_stepWidth, _selectedStepWidth * size),
      height: max(_stepHeight, _selectedStepHeight * size),
      child: DecoratedBox(
        decoration: BoxDecoration(
          color: ColorTween(begin: context.theme.primaryColorDark, end: context.colorScheme.primary).lerp(size),
          shape: BoxShape.rectangle,
          borderRadius: BorderRadius.circular(_stepHeight),
        ),
      ),
    );
  }
}
