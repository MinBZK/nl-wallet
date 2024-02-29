part of 'pin_bloc.dart';

sealed class PinState extends Equatable {
  const PinState();
}

class PinEntryInProgress extends PinState {
  final int enteredDigits;

  final bool afterBackspacePressed;

  const PinEntryInProgress(
    this.enteredDigits, {
    this.afterBackspacePressed = false,
  });

  @override
  List<Object> get props => [enteredDigits, afterBackspacePressed];
}

class PinValidateInProgress extends PinState {
  const PinValidateInProgress();

  @override
  List<Object> get props => [];
}

class PinValidateSuccess extends PinState {
  final String? returnUrl;

  const PinValidateSuccess({this.returnUrl});

  @override
  List<Object?> get props => [returnUrl];
}

class PinValidateFailure extends PinState {
  final int leftoverAttempts;
  final bool isFinalAttempt;

  const PinValidateFailure({required this.leftoverAttempts, this.isFinalAttempt = false});

  @override
  List<Object> get props => [leftoverAttempts, isFinalAttempt];
}

class PinValidateTimeout extends PinState {
  final DateTime expiryTime;

  const PinValidateTimeout(this.expiryTime);

  @override
  List<Object> get props => [expiryTime];
}

class PinValidateBlocked extends PinState {
  const PinValidateBlocked();

  @override
  List<Object> get props => [];
}

class PinValidateNetworkError extends PinState implements NetworkErrorState {
  @override
  final Object error;

  @override
  final bool hasInternet;

  @override
  final int? statusCode;

  const PinValidateNetworkError({required this.error, required this.hasInternet, this.statusCode});

  @override
  List<Object?> get props => [hasInternet, statusCode, error];
}

class PinValidateGenericError extends PinState implements ErrorState {
  @override
  final Object error;

  const PinValidateGenericError({required this.error});

  @override
  List<Object> get props => [error];
}
