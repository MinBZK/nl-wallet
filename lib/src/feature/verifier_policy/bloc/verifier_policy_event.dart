part of 'verifier_policy_bloc.dart';

abstract class VerifierPolicyEvent extends Equatable {
  const VerifierPolicyEvent();
}

class VerifierPolicyLoadTriggered extends VerifierPolicyEvent {
  final String sessionId;

  const VerifierPolicyLoadTriggered(this.sessionId);

  @override
  List<Object?> get props => [sessionId];
}
