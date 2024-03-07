part of 'setup_security_bloc.dart';

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
  double get stepperProgress => 0.24;

  @override
  List<Object?> get props => [enteredDigits, ...super.props];
}

class SetupSecuritySelectPinFailed extends SetupSecurityState {
  final PinValidationError reason;

  const SetupSecuritySelectPinFailed({required this.reason});

  @override
  double get stepperProgress => 0.24;
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
  double get stepperProgress => 0.32;
}

class SetupSecurityPinConfirmationFailed extends SetupSecurityState {
  final bool retryAllowed;

  const SetupSecurityPinConfirmationFailed({required this.retryAllowed});

  @override
  bool get canGoBack => true;

  @override
  double get stepperProgress => 0.32;
}

class SetupSecurityCreatingWallet extends SetupSecurityState {
  @override
  double get stepperProgress => 0.40;
}

class SetupSecurityCompleted extends SetupSecurityState {
  @override
  double get stepperProgress => 0.48;
}

class SetupSecurityGenericError extends SetupSecurityState implements ErrorState {
  @override
  double get stepperProgress => 0;

  @override
  final Object error;

  const SetupSecurityGenericError({required this.error});
}

class SetupSecurityNetworkError extends SetupSecurityState implements NetworkErrorState {
  @override
  double get stepperProgress => 0;

  @override
  final Object error;

  @override
  final int? statusCode;

  @override
  final bool hasInternet;

  const SetupSecurityNetworkError({required this.error, required this.hasInternet, this.statusCode});

  @override
  List<Object?> get props => [stepperProgress, hasInternet, statusCode, ...super.props];
}
