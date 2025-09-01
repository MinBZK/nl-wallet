import 'package:equatable/equatable.dart';

class FlowProgress extends Equatable {
  final int currentStep;
  final int totalSteps;

  double get progress => currentStep / totalSteps;

  const FlowProgress({
    required this.currentStep,
    required this.totalSteps,
  })  : assert(currentStep <= totalSteps, 'currentStep can never exceed totalSteps'),
        assert(currentStep > 0, 'This index is shown to the end-user and is thus 1-based');

  @override
  List<Object?> get props => [currentStep, totalSteps];
}
