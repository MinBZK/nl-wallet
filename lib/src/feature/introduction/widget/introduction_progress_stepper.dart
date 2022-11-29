import 'package:flutter/material.dart';

class IntroductionProgressStepper extends StatelessWidget {
  final int currentStep;
  final int totalSteps;

  const IntroductionProgressStepper({required this.currentStep, required this.totalSteps, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    List<Widget> steps = List.generate(totalSteps, (index) {
      if (currentStep != index) {
        return _buildStep(context);
      } else {
        return _buildCurrentStep(context);
      }
    });

    return Wrap(
      crossAxisAlignment: WrapCrossAlignment.center,
      spacing: 8,
      children: steps,
    );
  }

  Widget _buildStep(BuildContext context) {
    return SizedBox(
      width: 4,
      height: 4,
      child: DecoratedBox(
        decoration: BoxDecoration(
          color: Theme.of(context).primaryColorDark,
          shape: BoxShape.circle,
        ),
      ),
    );
  }

  Widget _buildCurrentStep(BuildContext context) {
    return SizedBox(
      width: 16,
      height: 6,
      child: DecoratedBox(
        decoration: BoxDecoration(
          color: Theme.of(context).primaryColor,
          shape: BoxShape.rectangle,
          borderRadius: BorderRadius.circular(3),
        ),
      ),
    );
  }
}
