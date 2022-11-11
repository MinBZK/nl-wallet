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
