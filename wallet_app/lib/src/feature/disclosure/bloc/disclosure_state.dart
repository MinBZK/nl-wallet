part of 'disclosure_bloc.dart';

sealed class DisclosureState extends Equatable {
  bool get showStopConfirmation => true;

  bool get canGoBack => false;

  bool get didGoBack => false;

  double get stepperProgress => 0.0;

  const DisclosureState();

  @override
  List<Object?> get props => [showStopConfirmation, canGoBack, didGoBack, stepperProgress];
}

class DisclosureInitial extends DisclosureState {
  const DisclosureInitial();
}

class DisclosureLoadInProgress extends DisclosureState {}

class DisclosureGenericError extends DisclosureState {
  @override
  bool get showStopConfirmation => false;
}

class DisclosureNetworkError extends DisclosureState {
  final bool hasInternet;

  const DisclosureNetworkError({this.hasInternet = true});

  @override
  bool get showStopConfirmation => false;

  @override
  List<Object?> get props => [hasInternet, ...super.props];
}

class DisclosureCheckOrganization extends DisclosureState {
  final Organization relyingParty;
  final bool isFirstInteractionWithOrganization;
  final bool afterBackPressed;

  @override
  double get stepperProgress => 0.25;

  @override
  bool get didGoBack => afterBackPressed;

  const DisclosureCheckOrganization({
    required this.relyingParty,
    required this.isFirstInteractionWithOrganization,
    this.afterBackPressed = false,
  });

  @override
  List<Object?> get props => [
        relyingParty,
        isFirstInteractionWithOrganization,
        ...super.props,
      ];
}

class DisclosureMissingAttributes extends DisclosureState {
  final Organization relyingParty;
  final List<Attribute> missingAttributes;

  @override
  double get stepperProgress => 0.5;

  @override
  bool get canGoBack => true;

  @override
  bool get showStopConfirmation => false;

  const DisclosureMissingAttributes({
    required this.relyingParty,
    required this.missingAttributes,
  });

  @override
  List<Object?> get props => [
        relyingParty,
        missingAttributes,
        ...super.props,
      ];
}

class DisclosureConfirmDataAttributes extends DisclosureState {
  final Organization relyingParty;
  final Map<WalletCard, List<DataAttribute>> requestedAttributes;
  final Policy policy;
  final LocalizedText requestPurpose;
  final bool afterBackPressed;

  @override
  double get stepperProgress => 0.5;

  @override
  bool get didGoBack => afterBackPressed;

  @override
  bool get canGoBack => true;

  const DisclosureConfirmDataAttributes({
    required this.relyingParty,
    required this.requestPurpose,
    required this.requestedAttributes,
    required this.policy,
    this.afterBackPressed = false,
  });

  @override
  List<Object?> get props => [
        relyingParty,
        requestPurpose,
        requestedAttributes,
        policy,
        ...super.props,
      ];
}

class DisclosureConfirmPin extends DisclosureState {
  const DisclosureConfirmPin();

  @override
  double get stepperProgress => 0.75;

  @override
  bool get canGoBack => true;
}

class DisclosureSuccess extends DisclosureState {
  final Organization relyingParty;
  final String? returnUrl;

  @override
  double get stepperProgress => 1;

  @override
  bool get showStopConfirmation => false;

  const DisclosureSuccess({required this.relyingParty, this.returnUrl});

  @override
  List<Object?> get props => [relyingParty, returnUrl, ...super.props];
}

class DisclosureStopped extends DisclosureState {
  @override
  double get stepperProgress => 1;

  @override
  bool get showStopConfirmation => false;

  const DisclosureStopped();
}

class DisclosureLeftFeedback extends DisclosureState {
  @override
  double get stepperProgress => 1;

  @override
  bool get showStopConfirmation => false;

  const DisclosureLeftFeedback();
}
