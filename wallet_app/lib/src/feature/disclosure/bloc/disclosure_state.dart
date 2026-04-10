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

/// Initial state where the session is being initialized and loading is shown.
class DisclosureInitial extends DisclosureState {
  @override
  bool get showStopConfirmation => true;

  const DisclosureInitial();
}

/// State indicating that session details are being loaded or attributes are being verified.
class DisclosureLoadInProgress extends DisclosureState {
  @override
  bool get showStopConfirmation => true;

  @override
  final FlowProgress? stepperProgress;

  const DisclosureLoadInProgress(this.stepperProgress);
}

/// Error state triggered when a disclosure QR code is scanned using an external camera app.
class DisclosureExternalScannerError extends DisclosureState implements ErrorState {
  @override
  final ApplicationError error;

  @override
  bool get showStopConfirmation => false;

  const DisclosureExternalScannerError({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}

/// State representing an unexpected generic error that occurred during the disclosure process.
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

/// Error state indicating a problem with the organization (relying party) requesting the data.
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

/// Error state shown when the disclosure session has timed out and needs to be restarted.
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

/// Error state emitted when the session was cancelled remotely by another device or the relying party.
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

/// Error state representing a network connectivity issue during the disclosure flow.
class DisclosureNetworkError extends DisclosureState implements NetworkErrorState {
  @override
  final NetworkError error;

  @override
  bool get showStopConfirmation => false;

  const DisclosureNetworkError({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}

/// State where the safety and origin of the request URL are verified for fraud prevention.
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

/// State prompting the user to approve the organization for a login request.
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

/// State indicating that the user does not possess the required attributes to satisfy the request.
class DisclosureMissingAttributes extends DisclosureState {
  final Organization relyingParty;
  final List<MissingAttribute> missingAttributes;

  @override
  final FlowProgress stepperProgress;

  @override
  bool get canGoBack => false;

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

/// State for user selection and confirmation of the specific attributes to be shared.
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

/// State prompting the user for their PIN to authorize and complete the data disclosure.
class DisclosureConfirmPin extends DisclosureState {
  final Organization relyingParty;
  final bool isLoginFlow;
  final List<int> selectedIndices;

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
    required this.selectedIndices,
    required bool isCrossDevice,
  }) : stepperProgress = _calculateStepperProgress(isCrossDevice: isCrossDevice);

  @override
  List<Object?> get props => [relyingParty, isLoginFlow, selectedIndices, ...super.props];
}

/// State indicating that the disclosure was successfully completed and the data was shared.
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

/// State shown when the user has intentionally stopped or cancelled the disclosure session.
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

/// State representing that the user has submitted feedback or a report after stopping the flow.
class DisclosureLeftFeedback extends DisclosureState {
  @override
  bool get showStopConfirmation => false;

  final String? returnUrl;

  const DisclosureLeftFeedback({this.returnUrl});
}

/// Error state emitted when a close-proximity connection is lost during the disclosure.
class DisclosureCloseProximityDisconnected extends DisclosureState {
  final bool isLoginFlow;

  @override
  bool get showStopConfirmation => false;

  const DisclosureCloseProximityDisconnected({required this.isLoginFlow});

  @override
  List<Object?> get props => [isLoginFlow, ...super.props];
}
