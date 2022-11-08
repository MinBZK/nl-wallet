part of 'verifier_policy_bloc.dart';

abstract class VerifierPolicyState extends Equatable {
  const VerifierPolicyState();
}

class VerifierPolicyInitial extends VerifierPolicyState {
  @override
  List<Object> get props => [];
}

class VerifierPolicyLoadInProgress extends VerifierPolicyState {
  @override
  List<Object> get props => [];
}

class VerifierPolicyLoadFailure extends VerifierPolicyState {
  final String sessionId;

  const VerifierPolicyLoadFailure(this.sessionId);

  @override
  List<Object> get props => [sessionId];
}

class VerifierPolicyLoadSuccess extends VerifierPolicyState {
  final VerifierPolicy policy;

  const VerifierPolicyLoadSuccess(this.policy);

  @override
  List<Object> get props => [policy];
}
