part of 'disclosure_bloc.dart';

sealed class DisclosureState extends Equatable {
  bool get showStopConfirmation => true;

  bool get canGoBack => false;

  bool get didGoBack => false;

  double get stepperProgress => 0.0;

  /// This 'flow' object is used when running mock builds of the app.
  DisclosureFlow? get flow => null;

  Organization? get organization => flow?.organization;

  const DisclosureState();

  @override
  List<Object?> get props => [showStopConfirmation, canGoBack, didGoBack, stepperProgress, flow];
}

class DisclosureInitial extends DisclosureState {
  const DisclosureInitial();
}

class DisclosureLoadInProgress extends DisclosureState {}

class DisclosureGenericError extends DisclosureState {
  @override
  bool get showStopConfirmation => false;
}

class DisclosureCheckOrganization extends DisclosureState {
  @override
  final DisclosureFlow? flow;
  final Organization relyingParty;
  final String requestPurpose;
  final bool isFirstInteractionWithOrganization;

  final bool afterBackPressed;

  const DisclosureCheckOrganization(this.relyingParty, this.requestPurpose, this.isFirstInteractionWithOrganization,
      {this.flow, this.afterBackPressed = false});

  // Support from [DisclosureFlow] for backwards/mock compatibility
  factory DisclosureCheckOrganization.fromFlow(DisclosureFlow flow, {afterBackPressed = false}) {
    return DisclosureCheckOrganization(
      flow.organization,
      flow.requestPurpose,
      !flow.hasPreviouslyInteractedWithOrganization,
      flow: flow,
      afterBackPressed: afterBackPressed,
    );
  }

  @override
  List<Object?> get props => [flow, ...super.props];

  @override
  double get stepperProgress => 0.25;

  @override
  bool get didGoBack => afterBackPressed;
}

class DisclosureMissingAttributes extends DisclosureState {
  @override
  final DisclosureFlow? flow;

  final Organization relyingParty;
  final List<Attribute> missingAttributes;

  const DisclosureMissingAttributes(this.relyingParty, this.missingAttributes, {this.flow});

  // Support from [DisclosureFlow] for backwards/mock compatibility
  factory DisclosureMissingAttributes.fromFlow(DisclosureFlow flow) {
    return DisclosureMissingAttributes(
      flow.organization,
      flow.missingAttributes,
      flow: flow,
    );
  }

  @override
  List<Object?> get props => [flow, relyingParty, missingAttributes, ...super.props];

  @override
  double get stepperProgress => 0.5;

  @override
  bool get canGoBack => true;

  @override
  bool get showStopConfirmation => false;
}

class DisclosureConfirmDataAttributes extends DisclosureState {
  @override
  final DisclosureFlow? flow;
  final Organization relyingParty;
  final Map<WalletCard, List<DataAttribute>> availableAttributes;
  final Policy policy;
  final bool afterBackPressed;

  const DisclosureConfirmDataAttributes(this.relyingParty, this.availableAttributes, this.policy,
      {this.flow, this.afterBackPressed = false});

  factory DisclosureConfirmDataAttributes.fromFlow(DisclosureFlow flow, {bool afterBackPressed = false}) {
    return DisclosureConfirmDataAttributes(
      flow.organization,
      flow.availableAttributes,
      flow.policy,
      flow: flow,
      afterBackPressed: afterBackPressed,
    );
  }

  @override
  List<Object?> get props => [flow, relyingParty, availableAttributes, policy, ...super.props];

  @override
  double get stepperProgress => 0.5;

  @override
  bool get didGoBack => afterBackPressed;

  @override
  bool get canGoBack => true;
}

class DisclosureConfirmPin extends DisclosureState {
  @override
  final DisclosureFlow? flow;

  const DisclosureConfirmPin({this.flow});

  factory DisclosureConfirmPin.fromFlow(DisclosureFlow flow) {
    return DisclosureConfirmPin(flow: flow);
  }

  @override
  List<Object?> get props => [flow, ...super.props];

  @override
  double get stepperProgress => 0.75;

  @override
  bool get canGoBack => true;
}

class DisclosureSuccess extends DisclosureState {
  @override
  final DisclosureFlow? flow;

  final Organization relyingParty;

  const DisclosureSuccess(this.relyingParty, {this.flow});

  factory DisclosureSuccess.fromFlow(DisclosureFlow flow) {
    return DisclosureSuccess(flow.organization, flow: flow);
  }

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
