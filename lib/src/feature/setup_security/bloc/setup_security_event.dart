part of 'setup_security_bloc.dart';

abstract class SetupSecurityEvent extends Equatable {
  const SetupSecurityEvent();

  @override
  List<Object?> get props => [];
}

class SetupSecurityBackPressed extends SetupSecurityEvent {}

class PinDigitPressed extends SetupSecurityEvent {
  final int digit;

  const PinDigitPressed(this.digit);

  @override
  List<Object?> get props => [digit];
}

class PinBackspacePressed extends SetupSecurityEvent {}
