part of 'issuance_bloc.dart';

abstract class IssuanceState extends Equatable {
  bool get canGoBack => false;

  const IssuanceState();
}

class IssuanceInitial extends IssuanceState {
  @override
  List<Object> get props => [];
}

class IssuanceLoadInProgress extends IssuanceState {
  @override
  List<Object> get props => [];
}

class IssuanceLoadFailure extends IssuanceState {
  @override
  List<Object> get props => [];
}

class IssuanceCheckOrganization extends IssuanceState {
  final IssuanceResponse response;

  Organization get organization => response.organization;

  const IssuanceCheckOrganization(this.response);

  @override
  List<Object> get props => [organization];
}

class IssuanceProofIdentity extends IssuanceState {
  final IssuanceResponse response;

  const IssuanceProofIdentity(this.response);

  @override
  List<Object> get props => [];

  @override
  bool get canGoBack => true;
}

class IssuanceProvidePin extends IssuanceState {
  @override
  List<Object> get props => [];
}

class IssuanceProvidePinSuccess extends IssuanceState {
  @override
  List<Object> get props => [];
}

class IssuanceProvidePinFailure extends IssuanceState {
  @override
  List<Object> get props => [];
}

class IssuanceCheckCardAttributes extends IssuanceState {
  @override
  List<Object> get props => [];
}

class IssuanceCardAdded extends IssuanceState {
  @override
  List<Object> get props => [];
}

class IssuanceStopped extends IssuanceState {
  @override
  List<Object> get props => [];
}

class IssuanceGenericError extends IssuanceState {
  @override
  List<Object> get props => [];
}

class IssuanceIdentityValidationFailure extends IssuanceState {
  @override
  List<Object> get props => [];
}
