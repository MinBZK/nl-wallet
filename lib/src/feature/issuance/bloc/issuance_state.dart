part of 'issuance_bloc.dart';

abstract class IssuanceState extends Equatable {
  bool get showStopConfirmation => true;

  bool get canGoBack => false;

  bool get didGoBack => false;

  double get stepperProgress => 0.0;

  IssuanceFlow? get flow => null;

  Organization? get organization => flow?.organization;

  const IssuanceState();

  @override
  List<Object?> get props => [showStopConfirmation, canGoBack, didGoBack, stepperProgress, flow];
}

class IssuanceInitial extends IssuanceState {}

class IssuanceLoadInProgress extends IssuanceState {}

class IssuanceLoadFailure extends IssuanceState {}

class IssuanceCheckOrganization extends IssuanceState {
  @override
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
  @override
  final IssuanceFlow flow;

  final bool afterBackPressed;

  @override
  Organization get organization => flow.organization;

  List<Attribute> get requestedAttributes => flow.attributes;

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
  @override
  final IssuanceFlow flow;

  const IssuanceProvidePin(this.flow);

  @override
  List<Object?> get props => [flow, ...super.props];

  @override
  bool get canGoBack => true;

  @override
  double get stepperProgress => 0.6;
}

class IssuanceCheckDataOffering extends IssuanceState {
  @override
  final IssuanceFlow flow;

  const IssuanceCheckDataOffering(this.flow);

  @override
  List<Object?> get props => [flow, ...super.props];

  @override
  double get stepperProgress => 0.8;
}

class IssuanceCardAdded extends IssuanceState {
  @override
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
