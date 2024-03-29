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

class DisclosureGenericError extends DisclosureState implements ErrorState {
  @override
  final Object error;

  @override
  bool get showStopConfirmation => false;

  const DisclosureGenericError({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}

class DisclosureNetworkError extends DisclosureState implements NetworkErrorState {
  @override
  bool get showStopConfirmation => false;

  @override
  final bool hasInternet;

  @override
  final Object error;

  @override
  final int? statusCode;

  const DisclosureNetworkError({
    this.statusCode,
    required this.error,
    this.hasInternet = true,
  });

  @override
  List<Object?> get props => [hasInternet, statusCode, error, ...super.props];
}

class DisclosureCheckOrganization extends DisclosureState {
  final Organization relyingParty;
  final String originUrl;
  final bool sharedDataWithOrganizationBefore;
  final DisclosureSessionType sessionType;
  final bool afterBackPressed;

  @override
  double get stepperProgress => 0.25;

  @override
  bool get didGoBack => afterBackPressed;

  const DisclosureCheckOrganization({
    required this.relyingParty,
    required this.originUrl,
    required this.sharedDataWithOrganizationBefore,
    required this.sessionType,
    this.afterBackPressed = false,
  });

  @override
  List<Object?> get props => [
        relyingParty,
        originUrl,
        sharedDataWithOrganizationBefore,
        sessionType,
        ...super.props,
      ];
}

class DisclosureCheckOrganizationForLogin extends DisclosureState {
  final Organization relyingParty;
  final String originUrl;
  final DisclosureSessionType sessionType;
  final bool sharedDataWithOrganizationBefore;
  final bool afterBackPressed;
  final Policy policy;
  final Map<WalletCard, List<DataAttribute>> requestedAttributes;

  @override
  double get stepperProgress => 0.25;

  @override
  bool get didGoBack => afterBackPressed;

  const DisclosureCheckOrganizationForLogin({
    required this.relyingParty,
    required this.originUrl,
    required this.sessionType,
    required this.policy,
    required this.requestedAttributes,
    required this.sharedDataWithOrganizationBefore,
    this.afterBackPressed = false,
  });

  @override
  List<Object?> get props => [
        relyingParty,
        originUrl,
        sessionType,
        policy,
        requestedAttributes,
        sharedDataWithOrganizationBefore,
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
  final Organization relyingParty;
  final bool isLoginFlow;

  const DisclosureConfirmPin({
    required this.relyingParty,
    this.isLoginFlow = false,
  });

  @override
  double get stepperProgress => 0.75;

  @override
  bool get canGoBack => true;

  @override
  List<Object?> get props => [relyingParty, isLoginFlow, ...super.props];
}

class DisclosureSuccess extends DisclosureState {
  final Organization relyingParty;
  final String? returnUrl;
  final bool isLoginFlow;

  @override
  double get stepperProgress => 1;

  @override
  bool get showStopConfirmation => false;

  const DisclosureSuccess({required this.relyingParty, this.returnUrl, this.isLoginFlow = false});

  @override
  List<Object?> get props => [relyingParty, returnUrl, isLoginFlow, ...super.props];
}

class DisclosureStopped extends DisclosureState {
  final Organization organization;

  @override
  double get stepperProgress => 1;

  @override
  bool get showStopConfirmation => false;

  const DisclosureStopped({required this.organization});

  @override
  List<Object?> get props => [organization, ...super.props];
}

class DisclosureLeftFeedback extends DisclosureState {
  @override
  double get stepperProgress => 1;

  @override
  bool get showStopConfirmation => false;

  const DisclosureLeftFeedback();
}
