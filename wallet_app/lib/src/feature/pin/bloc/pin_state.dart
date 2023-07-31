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
  const PinValidateSuccess();

  @override
  List<Object> get props => [];
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

class PinValidateServerError extends PinState {
  const PinValidateServerError();

  @override
  List<Object> get props => [];
}

class PinValidateGenericError extends PinState {
  const PinValidateGenericError();

  @override
  List<Object> get props => [];
}
