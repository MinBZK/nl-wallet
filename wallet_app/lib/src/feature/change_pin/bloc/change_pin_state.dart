part of 'change_pin_bloc.dart';

sealed class ChangePinState extends Equatable {
  const ChangePinState();

  bool get didGoBack => false;

  @override
  List<Object?> get props => [didGoBack];
}

class ChangePinInitial extends ChangePinState {
  @override
  final bool didGoBack;

  const ChangePinInitial({this.didGoBack = false});
}

class ChangePinSelectNewPinInProgress extends ChangePinState {
  final int enteredDigits;

  @override
  final bool didGoBack;

  final bool afterBackspacePressed;

  const ChangePinSelectNewPinInProgress(
    this.enteredDigits, {
    this.didGoBack = false,
    this.afterBackspacePressed = false,
  });

  @override
  List<Object?> get props => [enteredDigits, afterBackspacePressed, ...super.props];
}

class ChangePinSelectNewPinFailed extends ChangePinState {
  final PinValidationError reason;

  const ChangePinSelectNewPinFailed({required this.reason});

  @override
  List<Object?> get props => [reason, ...super.props];
}

class ChangePinConfirmNewPinInProgress extends ChangePinState {
  final int enteredDigits;

  final bool afterBackspacePressed;

  const ChangePinConfirmNewPinInProgress(this.enteredDigits, {this.afterBackspacePressed = false});

  @override
  List<Object?> get props => [enteredDigits, afterBackspacePressed, ...super.props];
}

class ChangePinConfirmNewPinFailed extends ChangePinState {
  final bool retryAllowed;

  const ChangePinConfirmNewPinFailed({required this.retryAllowed});
}

class ChangePinUpdating extends ChangePinState {}

class ChangePinCompleted extends ChangePinState {}

class ChangePinGenericError extends ChangePinState implements ErrorState {
  @override
  final ApplicationError error;

  const ChangePinGenericError({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}

class ChangePinNetworkError extends ChangePinState implements NetworkErrorState {
  @override
  final ApplicationError error;

  @override
  final bool hasInternet;

  @override
  final int? statusCode;

  const ChangePinNetworkError({required this.error, required this.hasInternet, this.statusCode});

  @override
  List<Object?> get props => [error, hasInternet, statusCode, ...super.props];
}
