part of 'disclosure_bloc.dart';

// Normal amount of steps for disclosure for the same device flow
const kDisclosureSteps = 3;

// The extra steps the user has to perform in cross device flows
const kExtraCrossDeviceSteps = 1;

sealed class DisclosureState extends Equatable {
  bool get showStopConfirmation => true;

  bool get canGoBack => false;

  bool get didGoBack => false;

  FlowProgress? get stepperProgress => null;

  const DisclosureState();

  @override
  List<Object?> get props => [showStopConfirmation, canGoBack, didGoBack, stepperProgress];
}

class DisclosureInitial extends DisclosureState {
  @override
  bool get showStopConfirmation => true;

  const DisclosureInitial();
}

class DisclosureLoadInProgress extends DisclosureState {
  @override
  bool get showStopConfirmation => true;

  @override
  final FlowProgress? stepperProgress;

  const DisclosureLoadInProgress(this.stepperProgress);
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

class DisclosureRelyingPartyError extends DisclosureState implements ErrorState {
  @override
  final ApplicationError error;

  final LocalizedText? organizationName;

  @override
  bool get showStopConfirmation => false;

  const DisclosureRelyingPartyError({required this.error, this.organizationName});

  @override
  List<Object?> get props => [error, organizationName, ...super.props];
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
class DisclosureSessionCancelled extends DisclosureState implements ErrorState {
  final Organization? relyingParty;
  final String? returnUrl;

  @override
  final ApplicationError error;

  @override
  bool get showStopConfirmation => false;

  const DisclosureSessionCancelled({
    required this.error,
    this.relyingParty,
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

class DisclosureCheckUrl extends DisclosureState {
  final String originUrl;
  final bool afterBackPressed;

  @override
  FlowProgress get stepperProgress =>
      const FlowProgress(currentStep: 1, totalSteps: kDisclosureSteps + kExtraCrossDeviceSteps);

  @override
  bool get didGoBack => afterBackPressed;

  const DisclosureCheckUrl({
    required this.originUrl,
    this.afterBackPressed = false,
  });

  @override
  List<Object?> get props => [originUrl, ...super.props];
}

class DisclosureCheckOrganizationForLogin extends DisclosureState {
  /// The organization requesting attributes for the login session.
  final Organization relyingParty;

  /// The URL origin of the login request.
  final String originUrl;

  /// List of card requests for the user to choose which card to disclose.
  final List<DiscloseCardRequest> cardRequests;

  /// The policy document outlining how the relying party handles disclosed data.
  final Policy policy;

  /// Type of session (i.e., cross-device or regular).
  final DisclosureSessionType sessionType;

  /// Indicates if the user has previously shared data with this organization.
  final bool sharedDataWithOrganizationBefore;

  /// Indicates if this state was reached by pressing the back button.
  final bool afterBackPressed;

  @override
  FlowProgress get stepperProgress => canGoBack
      ? const FlowProgress(currentStep: 2, totalSteps: kDisclosureSteps + kExtraCrossDeviceSteps)
      : const FlowProgress(currentStep: 1, totalSteps: kDisclosureSteps);

  @override
  bool get didGoBack => afterBackPressed;

  @override
  bool get canGoBack => sessionType == DisclosureSessionType.crossDevice;

  const DisclosureCheckOrganizationForLogin({
    required this.relyingParty,
    required this.originUrl,
    required this.sessionType,
    required this.policy,
    required this.cardRequests,
    required this.sharedDataWithOrganizationBefore,
    this.afterBackPressed = false,
  });

  @override
  List<Object?> get props => [
        relyingParty,
        originUrl,
        sessionType,
        policy,
        cardRequests,
        sharedDataWithOrganizationBefore,
        ...super.props,
      ];
}

class DisclosureMissingAttributes extends DisclosureState {
  final Organization relyingParty;
  final List<MissingAttribute> missingAttributes;

  @override
  final FlowProgress stepperProgress;

  @override
  bool get canGoBack => true;

  @override
  bool get showStopConfirmation => false;

  DisclosureMissingAttributes({
    required this.relyingParty,
    required this.missingAttributes,
    required bool isCrossDevice,
  }) : stepperProgress = FlowProgress(
          currentStep: isCrossDevice ? 2 : 1,
          totalSteps: kDisclosureSteps + (isCrossDevice ? kExtraCrossDeviceSteps : 0),
        );

  @override
  List<Object?> get props => [
        relyingParty,
        missingAttributes,
        ...super.props,
      ];
}

class DisclosureConfirmDataAttributes extends DisclosureState {
  /// The organization requesting access to the user's wallet data.
  final Organization relyingParty;

  /// A localized text describing the purpose of the data request.
  final LocalizedText requestPurpose;

  /// A list of card disclosure requests, each allowing the user to select which card to share.
  final List<DiscloseCardRequest> cardRequests;

  /// The relying party's policy document outlining data handling practices.
  final Policy policy;

  /// Indicates if this state was reached by pressing the back button.
  final bool afterBackPressed;

  /// Specifies the type of disclosure session (e.g., cross-device or regular).
  final DisclosureSessionType sessionType;

  @override
  FlowProgress get stepperProgress => sessionType == DisclosureSessionType.crossDevice
      ? const FlowProgress(currentStep: 2, totalSteps: kDisclosureSteps + kExtraCrossDeviceSteps)
      : const FlowProgress(currentStep: 1, totalSteps: kDisclosureSteps);

  @override
  bool get didGoBack => afterBackPressed;

  @override
  bool get canGoBack => sessionType == DisclosureSessionType.crossDevice;

  const DisclosureConfirmDataAttributes({
    required this.relyingParty,
    required this.requestPurpose,
    required this.cardRequests,
    required this.policy,
    required this.sessionType,
    this.afterBackPressed = false,
  });

  DisclosureConfirmDataAttributes copyWith({
    Organization? relyingParty,
    LocalizedText? requestPurpose,
    List<DiscloseCardRequest>? cardRequests,
    Policy? policy,
    DisclosureSessionType? sessionType,
    bool? afterBackPressed,
  }) {
    return DisclosureConfirmDataAttributes(
      relyingParty: relyingParty ?? this.relyingParty,
      requestPurpose: requestPurpose ?? this.requestPurpose,
      cardRequests: cardRequests ?? this.cardRequests,
      policy: policy ?? this.policy,
      sessionType: sessionType ?? this.sessionType,
      afterBackPressed: afterBackPressed ?? this.afterBackPressed,
    );
  }

  /// Returns a new [DisclosureConfirmDataAttributes] with the updated request.
  DisclosureConfirmDataAttributes updateWith(DiscloseCardRequest updatedEntry) {
    final updatedCardRequests = cardRequests.replace(updatedEntry, (it) => it.id);
    return copyWith(cardRequests: updatedCardRequests);
  }

  @override
  List<Object?> get props => [
        relyingParty,
        requestPurpose,
        cardRequests,
        policy,
        sessionType,
        ...super.props,
      ];
}

class DisclosureConfirmPin extends DisclosureState {
  final Organization relyingParty;
  final bool isLoginFlow;

  @override
  final FlowProgress stepperProgress;

  @override
  bool get canGoBack => true;

  static FlowProgress _calculateStepperProgress({required bool isCrossDevice}) {
    final extraSteps = isCrossDevice ? kExtraCrossDeviceSteps : 0;
    final totalSteps = kDisclosureSteps + extraSteps;
    final currentStep = 2 + extraSteps;
    return FlowProgress(currentStep: currentStep, totalSteps: totalSteps);
  }

  DisclosureConfirmPin({
    required this.relyingParty,
    this.isLoginFlow = false,
    required bool isCrossDevice,
  }) : stepperProgress = _calculateStepperProgress(isCrossDevice: isCrossDevice);

  @override
  List<Object?> get props => [relyingParty, isLoginFlow, ...super.props];
}

class DisclosureSuccess extends DisclosureState {
  final Organization relyingParty;
  final WalletEvent? event;
  final String? returnUrl;
  final bool isLoginFlow;

  @override
  final FlowProgress stepperProgress;

  @override
  bool get showStopConfirmation => false;

  static FlowProgress _calculateStepperProgress({required bool isCrossDevice}) {
    final extraSteps = isCrossDevice ? kExtraCrossDeviceSteps : 0;
    final totalSteps = kDisclosureSteps + extraSteps;
    return FlowProgress(currentStep: totalSteps, totalSteps: totalSteps);
  }

  DisclosureSuccess({
    required this.relyingParty,
    this.event,
    this.returnUrl,
    this.isLoginFlow = false,
    required bool isCrossDevice,
  }) : stepperProgress = _calculateStepperProgress(isCrossDevice: isCrossDevice);

  @override
  List<Object?> get props => [relyingParty, returnUrl, isLoginFlow, ...super.props];
}

class DisclosureStopped extends DisclosureState {
  final Organization organization;
  final bool isLoginFlow;
  final String? returnUrl;

  @override
  bool get showStopConfirmation => false;

  const DisclosureStopped({required this.organization, this.isLoginFlow = false, this.returnUrl});

  @override
  List<Object?> get props => [organization, isLoginFlow, returnUrl, ...super.props];
}

class DisclosureLeftFeedback extends DisclosureState {
  @override
  bool get showStopConfirmation => false;

  final String? returnUrl;

  const DisclosureLeftFeedback({this.returnUrl});
}
