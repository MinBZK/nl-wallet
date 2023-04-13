part of 'pin_bloc.dart';

abstract class PinEvent extends Equatable {
  const PinEvent();
}

class PinDigitPressed extends PinEvent {
  final int digit;

  const PinDigitPressed(this.digit);

  @override
  List<Object?> get props => [digit];
}

class PinBackspacePressed extends PinEvent {
  const PinBackspacePressed();

  @override
  List<Object?> get props => [];
}
