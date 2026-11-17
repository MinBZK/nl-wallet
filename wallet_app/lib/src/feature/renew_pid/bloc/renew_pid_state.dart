part of 'renew_pid_bloc.dart';

const kRenewPidSteps = 4;

sealed class RenewPidState extends Equatable {
  FlowProgress? get stepperProgress => null;

  bool get canGoBack => false;

  bool get didGoBack => false;

  const RenewPidState();

  @override
  List<Object?> get props => [stepperProgress, didGoBack, canGoBack];
}

class RenewPidInitial extends RenewPidState {
  @override
  final bool didGoBack;

  const RenewPidInitial({this.didGoBack = false});

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 1, totalSteps: kRenewPidSteps);
}

class RenewPidLoadingDigidUrl extends RenewPidState {
  const RenewPidLoadingDigidUrl();

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 1, totalSteps: kRenewPidSteps);
}

class RenewPidAwaitingDigidAuthentication extends RenewPidState {
  final String authUrl;

  const RenewPidAwaitingDigidAuthentication(this.authUrl);

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 1, totalSteps: kRenewPidSteps);

  @override
  List<Object?> get props => [...super.props, authUrl];
}

class RenewPidVerifyingDigidAuthentication extends RenewPidState {
  const RenewPidVerifyingDigidAuthentication();

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 1, totalSteps: kRenewPidSteps);
}

class RenewPidDigidMismatch extends RenewPidState {
  const RenewPidDigidMismatch();
}

class RenewPidStopped extends RenewPidState {
  const RenewPidStopped();
}

/// Asks for the PIN to retrieve the PID from an issuer that does not provide previews.
class RenewPidConfirmPinToRetrieve extends RenewPidState {
  const RenewPidConfirmPinToRetrieve();

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 2, totalSteps: kRenewPidSteps);
}

class RenewPidCheckData extends RenewPidState {
  final List<Attribute> availableAttributes;
  @override
  final bool didGoBack;

  /// True when [availableAttributes] are of the issued PID instead of a preview, in which case the PID is only added
  /// to the wallet after the user accepts it and confirms with their PIN again.
  final bool issuedForReview;

  const RenewPidCheckData({required this.availableAttributes, this.didGoBack = false, this.issuedForReview = false});

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 2, totalSteps: kRenewPidSteps);

  @override
  List<Object?> get props => [availableAttributes, issuedForReview, ...super.props];
}

class RenewPidConfirmPin extends RenewPidState {
  /// Used to return to [RenewPidCheckData] when user presses back
  final List<Attribute> attributes;

  /// True when the PID was already issued for review, so confirming with the PIN commits it to the wallet.
  final bool issuedForReview;

  const RenewPidConfirmPin(this.attributes, {this.issuedForReview = false});

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 3, totalSteps: kRenewPidSteps);

  @override
  bool get canGoBack => true;

  @override
  List<Object?> get props => [attributes, issuedForReview, ...super.props];
}

class RenewPidUpdatingCards extends RenewPidState {
  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 3, totalSteps: kRenewPidSteps);

  const RenewPidUpdatingCards();
}

class RenewPidSuccess extends RenewPidState {
  final List<WalletCard> addedCards;

  const RenewPidSuccess(this.addedCards);

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: kRenewPidSteps, totalSteps: kRenewPidSteps);

  @override
  List<Object?> get props => [addedCards, ...super.props];
}

class RenewPidDigidLoginCancelled extends RenewPidState {
  const RenewPidDigidLoginCancelled();
}

class RenewPidError extends RenewPidState implements ErrorState {
  @override
  final ApplicationError error;

  const RenewPidError({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}
