part of 'setup_security_bloc.dart';

const _kTotalSteps = 3;

sealed class SetupSecurityState extends Equatable {
  const SetupSecurityState();

  bool get canGoBack => false;

  bool get didGoBack => false;

  double get stepperProgress => 0;

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
  bool get didGoBack => afterBackPressed;

  @override
  double get stepperProgress => 1 / _kTotalSteps;

  @override
  List<Object?> get props => [enteredDigits, ...super.props];
}

class SetupSecuritySelectPinFailed extends SetupSecurityState {
  final PinValidationError? reason;

  const SetupSecuritySelectPinFailed({required this.reason});

  @override
  double get stepperProgress => 1 / _kTotalSteps;
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
  double get stepperProgress => 2 / _kTotalSteps;
}

class SetupSecurityPinConfirmationFailed extends SetupSecurityState {
  final bool retryAllowed;

  const SetupSecurityPinConfirmationFailed({required this.retryAllowed});

  @override
  bool get canGoBack => true;

  @override
  double get stepperProgress => 2 / _kTotalSteps;
}

class SetupSecurityCreatingWallet extends SetupSecurityState {
  @override
  double get stepperProgress => 2.5 / _kTotalSteps;
}

class SetupSecurityCompleted extends SetupSecurityState {
  @override
  double get stepperProgress => 3 / _kTotalSteps;
}

class SetupSecurityGenericError extends SetupSecurityState {
  @override
  double get stepperProgress => 0;
}

class SetupSecurityNetworkError extends SetupSecurityState implements NetworkError {
  @override
  double get stepperProgress => 0;

  @override
  final int? statusCode;

  @override
  final bool hasInternet;

  const SetupSecurityNetworkError({required this.hasInternet, this.statusCode});

  @override
  List<Object?> get props => [stepperProgress, hasInternet, statusCode, ...super.props];
}
