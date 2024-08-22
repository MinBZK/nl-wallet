part of 'change_pin_bloc.dart';

abstract class ChangePinEvent extends Equatable {
  const ChangePinEvent();

  @override
  List<Object?> get props => [];
}

class ChangePinCurrentPinValidated extends ChangePinEvent {
  final String currentPin;

  const ChangePinCurrentPinValidated(this.currentPin);

  @override
  List<Object?> get props => [currentPin, ...super.props];
}

class ChangePinBackPressed extends ChangePinEvent {}

class PinDigitPressed extends ChangePinEvent {
  final int digit;

  const PinDigitPressed(this.digit);

  @override
  List<Object?> get props => [digit, ...super.props];
}

class PinBackspacePressed extends ChangePinEvent {}

class PinClearPressed extends ChangePinEvent {}

class ChangePinRetryPressed extends ChangePinEvent {}
