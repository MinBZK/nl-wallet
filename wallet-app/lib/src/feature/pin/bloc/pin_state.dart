part of 'pin_bloc.dart';

abstract class PinState extends Equatable {
  const PinState();
}

class PinEntryInProgress extends PinState {
  final int enteredDigits;

  const PinEntryInProgress(this.enteredDigits);

  @override
  List<Object> get props => [enteredDigits];
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

  const PinValidateFailure(this.leftoverAttempts);

  @override
  List<Object> get props => [leftoverAttempts];
}

class PinValidateBlocked extends PinState {
  const PinValidateBlocked();

  @override
  List<Object> get props => [];
}
