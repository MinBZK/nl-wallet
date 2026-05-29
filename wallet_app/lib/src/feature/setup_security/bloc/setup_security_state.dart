part of 'setup_security_bloc.dart';

sealed class SetupSecurityState extends Equatable {
  const SetupSecurityState();

  bool get canGoBack => false;

  bool get didGoBack => false;

  FlowProgress? get stepperProgress => null;

  @override
  List<Object?> get props => [canGoBack, didGoBack, stepperProgress];
}

class SetupSecuritySelectPinInProgress extends SetupSecurityState {
  final int enteredDigits;

  final bool afterBackPressed;

  final bool afterBackspacePressed;

  const SetupSecuritySelectPinInProgress(
    this.enteredDigits, {
    this.afterBackPressed = false,
    this.afterBackspacePressed = false,
  });

  @override
  bool get canGoBack => true;

  @override
  bool get didGoBack => afterBackPressed;

  @override
  FlowProgress get stepperProgress => FlowProgress(currentStep: 2, totalSteps: OnboardingHelper.totalSteps);

  @override
  List<Object?> get props => [enteredDigits, ...super.props];
}

class SetupSecuritySelectPinFailed extends SetupSecurityState {
  final PinValidationError reason;

  const SetupSecuritySelectPinFailed({required this.reason});

  @override
  FlowProgress get stepperProgress => FlowProgress(currentStep: 2, totalSteps: OnboardingHelper.totalSteps);

  @override
  List<Object?> get props => [reason, ...super.props];
}

class SetupSecurityPinConfirmationInProgress extends SetupSecurityState {
  final int enteredDigits;

  final bool afterBackspacePressed;

  const SetupSecurityPinConfirmationInProgress(this.enteredDigits, {this.afterBackspacePressed = false});

  @override
  bool get canGoBack => true;

  @override
  List<Object?> get props => [enteredDigits, ...super.props];

  @override
  FlowProgress get stepperProgress => FlowProgress(currentStep: 3, totalSteps: OnboardingHelper.totalSteps);
}

class SetupSecurityPinConfirmationFailed extends SetupSecurityState {
  final bool retryAllowed;

  const SetupSecurityPinConfirmationFailed({required this.retryAllowed});

  @override
  bool get canGoBack => true;

  @override
  FlowProgress get stepperProgress => FlowProgress(currentStep: 3, totalSteps: OnboardingHelper.totalSteps);
}

class SetupSecurityCreatingWallet extends SetupSecurityState {
  const SetupSecurityCreatingWallet();
}

class SetupSecurityConfigureBiometrics extends SetupSecurityState {
  final Biometrics biometrics;

  const SetupSecurityConfigureBiometrics({required this.biometrics})
    : assert(biometrics != Biometrics.none, 'This state is invalid without supported biometrics');

  @override
  FlowProgress get stepperProgress => FlowProgress(currentStep: 4, totalSteps: OnboardingHelper.totalSteps);

  @override
  List<Object?> get props => [biometrics, ...super.props];
}

class SetupSecurityCompleted extends SetupSecurityState {
  final Biometrics enabledBiometrics;

  const SetupSecurityCompleted({this.enabledBiometrics = Biometrics.none});

  @override
  FlowProgress get stepperProgress =>
      FlowProgress(currentStep: OnboardingHelper.totalSteps - 5, totalSteps: OnboardingHelper.totalSteps);

  @override
  List<Object?> get props => [enabledBiometrics, ...super.props];
}

class SetupSecurityError extends SetupSecurityState implements ErrorState {
  @override
  final ApplicationError error;

  const SetupSecurityError({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}
