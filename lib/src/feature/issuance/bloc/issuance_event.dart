part of 'issuance_bloc.dart';

abstract class IssuanceEvent extends Equatable {
  const IssuanceEvent();
}

class IssuanceLoadTriggered extends IssuanceEvent {
  final String sessionId;

  const IssuanceLoadTriggered(this.sessionId);

  @override
  List<Object?> get props => [sessionId];
}

class IssuanceVerifierApproved extends IssuanceEvent {
  const IssuanceVerifierApproved();

  @override
  List<Object?> get props => [];
}

class IssuanceBackPressed extends IssuanceEvent {
  const IssuanceBackPressed();

  @override
  List<Object?> get props => [];
}

class IssuanceVerifierDeclined extends IssuanceEvent {
  const IssuanceVerifierDeclined();

  @override
  List<Object?> get props => [];
}
