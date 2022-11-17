part of 'issuance_bloc.dart';

abstract class IssuanceState extends Equatable {
  bool get canGoBack => false;

  bool get didGoBack => false;

  double get stepperProgress => 0.0;

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
  final bool afterBackPressed;

  Organization get organization => response.organization;

  const IssuanceCheckOrganization(this.response, {this.afterBackPressed = false});

  @override
  List<Object> get props => [organization];

  @override
  double get stepperProgress => 0.2;

  @override
  bool get didGoBack => afterBackPressed;
}

class IssuanceProofIdentity extends IssuanceState {
  final IssuanceResponse response;
  final bool afterBackPressed;

  Organization get organization => response.organization;

  List<DataAttribute> get requestedAttributes => response.requestedAttributes;

  const IssuanceProofIdentity(this.response, {this.afterBackPressed = false});

  @override
  List<Object> get props => [response];

  @override
  bool get canGoBack => true;

  @override
  bool get didGoBack => afterBackPressed;

  @override
  double get stepperProgress => 0.4;
}

class IssuanceProvidePin extends IssuanceState {
  final IssuanceResponse response;

  const IssuanceProvidePin(this.response);

  @override
  List<Object> get props => [response];

  @override
  bool get canGoBack => true;

  @override
  double get stepperProgress => 0.6;
}

class IssuanceCheckDataOffering extends IssuanceState {
  final IssuanceResponse response;

  const IssuanceCheckDataOffering(this.response);

  @override
  List<Object> get props => [response];

  @override
  double get stepperProgress => 0.8;
}

class IssuanceCardAdded extends IssuanceState {
  final IssuanceResponse response;

  const IssuanceCardAdded(this.response);

  @override
  List<Object> get props => [response];
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
