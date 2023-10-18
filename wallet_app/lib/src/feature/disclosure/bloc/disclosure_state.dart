part of 'disclosure_bloc.dart';

sealed class DisclosureState extends Equatable {
  bool get showStopConfirmation => true;

  bool get canGoBack => false;

  bool get didGoBack => false;

  double get stepperProgress => 0.0;

  DisclosureFlow? get flow => null;

  Organization? get organization => flow?.organization;

  const DisclosureState();

  @override
  List<Object?> get props => [showStopConfirmation, canGoBack, didGoBack, stepperProgress, flow];
}

class DisclosureInitial extends DisclosureState {}

class DisclosureLoadInProgress extends DisclosureState {}

class DisclosureGenericError extends DisclosureState {
  @override
  bool get showStopConfirmation => false;
}

class DisclosureCheckOrganization extends DisclosureState {
  @override
  final DisclosureFlow flow;

  final bool afterBackPressed;

  const DisclosureCheckOrganization(this.flow, {this.afterBackPressed = false});

  @override
  List<Object?> get props => [flow, ...super.props];

  @override
  double get stepperProgress => 0.25;

  @override
  bool get didGoBack => afterBackPressed;
}

class DisclosureMissingAttributes extends DisclosureState {
  @override
  final DisclosureFlow flow;

  const DisclosureMissingAttributes(this.flow);

  @override
  List<Object?> get props => [flow, ...super.props];

  @override
  double get stepperProgress => 0.5;

  @override
  bool get canGoBack => true;

  @override
  bool get showStopConfirmation => false;
}

class DisclosureConfirmDataAttributes extends DisclosureState {
  @override
  final DisclosureFlow flow;

  final bool afterBackPressed;

  const DisclosureConfirmDataAttributes(this.flow, {this.afterBackPressed = false});

  @override
  List<Object?> get props => [flow, ...super.props];

  @override
  double get stepperProgress => 0.5;

  @override
  bool get didGoBack => afterBackPressed;

  @override
  bool get canGoBack => true;
}

class DisclosureConfirmPin extends DisclosureState {
  @override
  final DisclosureFlow flow;

  const DisclosureConfirmPin(this.flow);

  @override
  List<Object?> get props => [flow, ...super.props];

  @override
  double get stepperProgress => 0.75;

  @override
  bool get canGoBack => true;
}

class DisclosureSuccess extends DisclosureState {
  @override
  final DisclosureFlow flow;

  const DisclosureSuccess(this.flow);

  @override
  List<Object?> get props => [flow, ...super.props];

  @override
  double get stepperProgress => 1;

  @override
  bool get showStopConfirmation => false;
}

class DisclosureStopped extends DisclosureState {
  const DisclosureStopped();

  @override
  double get stepperProgress => 1;

  @override
  bool get showStopConfirmation => false;
}

class DisclosureLeftFeedback extends DisclosureState {
  const DisclosureLeftFeedback();

  @override
  double get stepperProgress => 1;

  @override
  bool get showStopConfirmation => false;
}
