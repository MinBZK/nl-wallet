part of 'issuance_bloc.dart';

abstract class IssuanceState extends Equatable {
  bool get showStopConfirmation => true;

  bool get canGoBack => false;

  bool get didGoBack => false;

  double get stepperProgress => 0.0;

  Organization? get organization => null;

  const IssuanceState();

  @override
  List<Object?> get props => [showStopConfirmation, canGoBack, didGoBack, stepperProgress, organization];
}

class IssuanceInitial extends IssuanceState {}

class IssuanceLoadInProgress extends IssuanceState {}

class IssuanceLoadFailure extends IssuanceState {}

class IssuanceCheckOrganization extends IssuanceState {
  final IssuanceFlow flow;
  final bool afterBackPressed;

  @override
  Organization get organization => flow.organization;

  const IssuanceCheckOrganization(this.flow, {this.afterBackPressed = false});

  @override
  List<Object?> get props => [flow, ...super.props];

  @override
  double get stepperProgress => 0.2;

  @override
  bool get didGoBack => afterBackPressed;
}

class IssuanceProofIdentity extends IssuanceState {
  final IssuanceFlow flow;
  final bool afterBackPressed;

  @override
  Organization get organization => flow.organization;

  List<DataAttribute> get requestedAttributes => flow.requestedDataAttributes;

  const IssuanceProofIdentity(this.flow, {this.afterBackPressed = false});

  @override
  List<Object?> get props => [flow, ...super.props];

  @override
  bool get canGoBack => true;

  @override
  bool get didGoBack => afterBackPressed;

  @override
  double get stepperProgress => 0.4;
}

class IssuanceProvidePin extends IssuanceState {
  final IssuanceFlow flow;

  const IssuanceProvidePin(this.flow);

  @override
  Organization get organization => flow.organization;

  @override
  List<Object?> get props => [flow, ...super.props];

  @override
  bool get canGoBack => true;

  @override
  double get stepperProgress => 0.6;
}

class IssuanceCheckDataOffering extends IssuanceState {
  final IssuanceFlow flow;

  const IssuanceCheckDataOffering(this.flow);

  @override
  List<Object?> get props => [flow, ...super.props];

  @override
  double get stepperProgress => 0.8;

  @override
  Organization get organization => flow.organization;
}

class IssuanceCardAdded extends IssuanceState {
  final IssuanceFlow flow;

  const IssuanceCardAdded(this.flow);

  @override
  List<Object?> get props => [flow, ...super.props];

  @override
  bool get showStopConfirmation => false;
}

class IssuanceStopped extends IssuanceState {
  @override
  List<Object> get props => [];

  @override
  bool get showStopConfirmation => false;
}

class IssuanceGenericError extends IssuanceState {
  @override
  List<Object> get props => [];

  @override
  bool get showStopConfirmation => false;
}

class IssuanceIdentityValidationFailure extends IssuanceState {
  @override
  List<Object> get props => [];

  @override
  bool get showStopConfirmation => false;
}
