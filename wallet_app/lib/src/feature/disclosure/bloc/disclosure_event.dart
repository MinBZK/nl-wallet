part of 'disclosure_bloc.dart';

/// Base class for all events handled by the [DisclosureBloc].
abstract class DisclosureEvent extends Equatable {
  const DisclosureEvent();

  @override
  List<Object?> get props => [];
}

/// Initiates a disclosure session from a URI (QR code or deep link).
///
/// Cancels any ongoing session and triggers [StartDisclosureUseCase].
/// Results in [DisclosureCheckUrl], [DisclosureConfirmDataAttributes],
/// [DisclosureCheckOrganizationForLogin], or [DisclosureMissingAttributes].
class DisclosureSessionStarted extends DisclosureEvent {
  final String uri;
  final bool isQrCode;

  const DisclosureSessionStarted(this.uri, {this.isQrCode = false});

  @override
  List<Object?> get props => [uri, isQrCode];
}

/// User approves the origin URL in a cross-device flow.
///
/// Transitions to attribute confirmation or missing attributes state.
class DisclosureUrlApproved extends DisclosureEvent {
  const DisclosureUrlApproved();
}

/// User confirms the selection of cards and attributes to be shared.
///
/// Transitions the flow to [DisclosureConfirmPin].
class DisclosureShareRequestedCardsApproved extends DisclosureEvent {
  const DisclosureShareRequestedCardsApproved();
}

/// User selects a different card for a specific attribute request.
///
/// Updates the BLoC's selection cache and refreshes the current state.
class DisclosureAlternativeCardSelected extends DisclosureEvent {
  final DiscloseCardRequest updatedCardRequest;

  const DisclosureAlternativeCardSelected(this.updatedCardRequest);

  @override
  List<Object?> get props => [updatedCardRequest];
}

/// User successfully entered their PIN and confirmed the disclosure.
///
/// Transitions to [DisclosureSuccess] and retrieves the resulting wallet event.
class DisclosurePinConfirmed extends DisclosureEvent {
  final String? returnUrl;

  const DisclosurePinConfirmed({this.returnUrl});

  @override
  List<Object?> get props => [returnUrl];
}

/// User navigates back to a previous step in the disclosure flow.
class DisclosureBackPressed extends DisclosureEvent {
  const DisclosureBackPressed();
}

/// Explicitly stops the disclosure session and shows the [DisclosureStopped] screen.
class DisclosureStopRequested extends DisclosureEvent {
  const DisclosureStopRequested();
}

/// Silently cancels the disclosure session (e.g., when dismissing the flow).
///
/// Does not trigger a state transition; intended for cleanup.
class DisclosureCancelRequested extends DisclosureEvent {
  const DisclosureCancelRequested();
}

/// User opts to report an issue, canceling the session and moving to [DisclosureLeftFeedback].
class DisclosureReportPressed extends DisclosureEvent {
  final ReportingOption option;

  const DisclosureReportPressed({required this.option});

  @override
  List<Object?> get props => [option];
}

/// PIN confirmation failed, triggering error handling and potential state change.
class DisclosureConfirmPinFailed extends DisclosureEvent {
  final ApplicationError error;

  const DisclosureConfirmPinFailed({required this.error});

  @override
  List<Object?> get props => [error];
}
