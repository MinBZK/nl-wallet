import 'package:equatable/equatable.dart';

class FlowProgress extends Equatable {
  final int currentStep;
  final int totalSteps;

  double get progress => currentStep / totalSteps;

  const FlowProgress({
    required this.currentStep,
    required this.totalSteps,
  }) : assert(currentStep <= totalSteps, 'currentStep can never exceed totalSteps');

  @override
  List<Object?> get props => [currentStep, totalSteps];
}
