import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_constants.dart';

const Size kSingleIndicatorSize = Size(12, 4);
const double kMarginWidth = 12;
const BorderRadius kBorderRadius = BorderRadius.all(Radius.circular(8));

class StepperIndicator extends StatelessWidget {
  final EdgeInsets padding;

  final int currentStep, totalSteps;

  double get progress => currentStep / totalSteps;

  int get stepsLeft => totalSteps - currentStep;

  const StepperIndicator({
    this.padding = const EdgeInsets.symmetric(horizontal: 16, vertical: 0.5),
    this.currentStep = 1,
    this.totalSteps = 5,
    super.key,
  }) : assert(
            (totalSteps - currentStep) < 10,
            'This component was not developed with infinite steps in mind, '
            'when reaching this threshold the design might have to be re-considered');

  @override
  Widget build(BuildContext context) {
    return Semantics(
      attributedLabel: context.l10n.generalWCAGStepper(currentStep, totalSteps).toAttributedString(context),
      excludeSemantics: true,
      child: Hero(
        tag: 'stepper_indicator',
        child: Padding(
          padding: padding,
          child: SizedBox(
            height: 4,
            child: LayoutBuilder(
              builder: (context, constraints) {
                return Stack(
                  fit: StackFit.passthrough,
                  children: [
                    ...List.generate(stepsLeft + 1, (i) => i).map(
                      (i) => _buildStepIndicator(context, i),
                    ),
                    _buildMainProgressIndicator(context),
                  ],
                );
              },
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildMainProgressIndicator(BuildContext context) {
    return AnimatedPositioned(
      curve: Curves.easeInOutCirc,
      left: 0,
      top: 0,
      right: stepsLeft * kSingleIndicatorSize.width + stepsLeft * kMarginWidth,
      height: 4,
      duration: kDefaultAnimationDuration,
      child: Container(
        decoration: BoxDecoration(
          color: context.colorScheme.primary,
          borderRadius: kBorderRadius,
        ),
      ),
    );
  }

  Widget _buildStepIndicator(BuildContext context, int i) {
    return Positioned(
      top: 0,
      width: kSingleIndicatorSize.width,
      height: kSingleIndicatorSize.height,
      right: i * kSingleIndicatorSize.width + i * kMarginWidth,
      child: _SingleStepIndicator(
        visible: i < stepsLeft,
        key: ValueKey(i),
      ),
    );
  }
}

class _SingleStepIndicator extends StatelessWidget {
  final bool visible;

  const _SingleStepIndicator({required this.visible, super.key});

  @override
  Widget build(BuildContext context) {
    return AnimatedContainer(
      curve: Curves.easeInOutCirc,
      duration: kDefaultAnimationDuration,
      decoration: BoxDecoration(
        color: context.colorScheme.primary.withValues(alpha: visible ? 1 : 0),
        borderRadius: kBorderRadius,
      ),
    );
  }
}
