part of 'setup_security_bloc.dart';

sealed class SetupSecurityState extends Equatable {
  const SetupSecurityState();

  bool get canGoBack => false;

  bool get didGoBack => false;

  FlowProgress get stepperProgress => const FlowProgress(currentStep: 3, totalSteps: kSetupSteps);

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
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 3, totalSteps: kSetupSteps);

  @override
  List<Object?> get props => [enteredDigits, ...super.props];
}

class SetupSecuritySelectPinFailed extends SetupSecurityState {
  final PinValidationError reason;

  const SetupSecuritySelectPinFailed({required this.reason});

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 3, totalSteps: kSetupSteps);

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
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 4, totalSteps: kSetupSteps);
}

class SetupSecurityPinConfirmationFailed extends SetupSecurityState {
  final bool retryAllowed;

  const SetupSecurityPinConfirmationFailed({required this.retryAllowed});

  @override
  bool get canGoBack => true;

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 4, totalSteps: kSetupSteps);
}

class SetupSecurityCreatingWallet extends SetupSecurityState {
  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 4, totalSteps: kSetupSteps);
}

class SetupSecurityConfigureBiometrics extends SetupSecurityState {
  final Biometrics biometrics;

  const SetupSecurityConfigureBiometrics({required this.biometrics})
      : assert(biometrics != Biometrics.none, 'This state is invalid without supported biometrics');

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 4, totalSteps: kSetupSteps);

  @override
  List<Object?> get props => [biometrics, ...super.props];
}

class SetupSecurityCompleted extends SetupSecurityState {
  final bool biometricsEnabled;

  const SetupSecurityCompleted({this.biometricsEnabled = false});

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 5, totalSteps: kSetupSteps);

  @override
  List<Object?> get props => [biometricsEnabled, ...super.props];
}

class SetupSecurityGenericError extends SetupSecurityState implements ErrorState {
  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 4, totalSteps: kSetupSteps);

  @override
  final Object error;

  const SetupSecurityGenericError({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}

class SetupSecurityDeviceIncompatibleError extends SetupSecurityState implements ErrorState {
  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 4, totalSteps: kSetupSteps);

  @override
  final Object error;

  const SetupSecurityDeviceIncompatibleError({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}

class SetupSecurityNetworkError extends SetupSecurityState implements NetworkErrorState {
  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 4, totalSteps: kSetupSteps);

  @override
  final Object error;

  @override
  final int? statusCode;

  @override
  final bool hasInternet;

  const SetupSecurityNetworkError({required this.error, required this.hasInternet, this.statusCode});

  @override
  List<Object?> get props => [error, stepperProgress, hasInternet, statusCode, ...super.props];
}
