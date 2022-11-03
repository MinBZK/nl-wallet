part of 'verification_bloc.dart';

abstract class VerificationEvent extends Equatable {
  const VerificationEvent();
}

class VerificationLoadRequested extends VerificationEvent {
  final String sessionId;

  const VerificationLoadRequested(this.sessionId);

  @override
  List<Object?> get props => [sessionId];
}
