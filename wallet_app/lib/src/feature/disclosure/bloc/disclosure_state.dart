part of 'disclosure_bloc.dart';

const kNormalDisclosureSteps = 4;
const kLoginDisclosureSteps = 3;

sealed class DisclosureState extends Equatable {
  bool get showStopConfirmation => true;

  bool get canGoBack => false;

  bool get didGoBack => false;

  FlowProgress get stepperProgress => const FlowProgress(currentStep: 0, totalSteps: 4);

  const DisclosureState();

  @override
  List<Object?> get props => [showStopConfirmation, canGoBack, didGoBack, stepperProgress];
}

class DisclosureInitial extends DisclosureState {
  @override
  bool get showStopConfirmation => false;

  const DisclosureInitial();
}

class DisclosureLoadInProgress extends DisclosureState {
  @override
  bool get showStopConfirmation => true;
}

/// This [ErrorState] is emitted when the user scanned a Disclosure QR with an external app (e.g. the built-in camera)
class DisclosureExternalScannerError extends DisclosureState implements ErrorState {
  @override
  final ApplicationError error;

  @override
  bool get showStopConfirmation => false;

  const DisclosureExternalScannerError({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}

class DisclosureGenericError extends DisclosureState implements ErrorState {
  @override
  final ApplicationError error;

  final String? returnUrl;

  @override
  bool get showStopConfirmation => false;

  const DisclosureGenericError({required this.error, this.returnUrl});

  @override
  List<Object?> get props => [error, ...super.props];
}

class DisclosureSessionExpired extends DisclosureState implements ErrorState {
  @override
  final ApplicationError error;

  @override
  bool get showStopConfirmation => false;

  final bool isCrossDevice;

  final bool canRetry;

  final String? returnUrl;

  const DisclosureSessionExpired({
    required this.error,
    required this.isCrossDevice,
    required this.canRetry,
    this.returnUrl,
  });

  @override
  List<Object?> get props => [error, canRetry, isCrossDevice, returnUrl, ...super.props];
}

/// State that is exposed when the session has been stopped remotely (e.g. the user pressed stop in wallet_web)
class DisclosureCancelledSessionError extends DisclosureState implements ErrorState {
  final Organization relyingParty;
  final String? returnUrl;

  @override
  final ApplicationError error;

  @override
  bool get showStopConfirmation => false;

  const DisclosureCancelledSessionError({
    required this.error,
    required this.relyingParty,
    this.returnUrl,
  });

  @override
  List<Object?> get props => [error, relyingParty, returnUrl, ...super.props];
}

class DisclosureNetworkError extends DisclosureState implements NetworkErrorState {
  @override
  bool get showStopConfirmation => false;

  @override
  final bool hasInternet;

  @override
  final ApplicationError error;

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
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 1, totalSteps: kNormalDisclosureSteps);

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
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 1, totalSteps: kLoginDisclosureSteps);

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
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 2, totalSteps: kNormalDisclosureSteps);

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
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 2, totalSteps: kNormalDisclosureSteps);

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
  FlowProgress get stepperProgress => FlowProgress(
        currentStep: isLoginFlow ? 2 : 3,
        totalSteps: isLoginFlow ? kLoginDisclosureSteps : kNormalDisclosureSteps,
      );

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
  FlowProgress get stepperProgress =>
      const FlowProgress(currentStep: kNormalDisclosureSteps, totalSteps: kNormalDisclosureSteps);

  @override
  bool get showStopConfirmation => false;

  const DisclosureSuccess({required this.relyingParty, this.returnUrl, this.isLoginFlow = false});

  @override
  List<Object?> get props => [relyingParty, returnUrl, isLoginFlow, ...super.props];
}

class DisclosureStopped extends DisclosureState {
  final Organization organization;
  final bool isLoginFlow;
  final String? returnUrl;

  @override
  FlowProgress get stepperProgress => FlowProgress(
        currentStep: isLoginFlow ? kLoginDisclosureSteps : kNormalDisclosureSteps,
        totalSteps: isLoginFlow ? kLoginDisclosureSteps : kNormalDisclosureSteps,
      );

  @override
  bool get showStopConfirmation => false;

  const DisclosureStopped({required this.organization, this.isLoginFlow = false, this.returnUrl});

  @override
  List<Object?> get props => [organization, isLoginFlow, returnUrl, ...super.props];
}

class DisclosureLeftFeedback extends DisclosureState {
  @override
  FlowProgress get stepperProgress =>
      const FlowProgress(currentStep: kNormalDisclosureSteps, totalSteps: kNormalDisclosureSteps);

  @override
  bool get showStopConfirmation => false;

  final String? returnUrl;

  const DisclosureLeftFeedback({this.returnUrl});
}
