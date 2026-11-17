part of 'wallet_personalize_bloc.dart';

sealed class WalletPersonalizeState extends Equatable {
  FlowProgress? get stepperProgress => null;

  bool get canGoBack => false;

  bool get didGoBack => false;

  const WalletPersonalizeState();

  @override
  List<Object?> get props => [stepperProgress, didGoBack, canGoBack];
}

class WalletPersonalizeInitial extends WalletPersonalizeState {
  @override
  final bool didGoBack;

  const WalletPersonalizeInitial({this.didGoBack = false});

  @override
  FlowProgress get stepperProgress =>
      FlowProgress(currentStep: OnboardingHelper.totalSteps - 3, totalSteps: OnboardingHelper.totalSteps);
}

class WalletPersonalizeLoadingIssuanceUrl extends WalletPersonalizeState {
  const WalletPersonalizeLoadingIssuanceUrl();
}

class WalletPersonalizeConnectDigid extends WalletPersonalizeState {
  final String authUrl;

  const WalletPersonalizeConnectDigid(this.authUrl);

  @override
  List<Object?> get props => [...super.props, authUrl];
}

class WalletPersonalizeAuthenticating extends WalletPersonalizeState {
  const WalletPersonalizeAuthenticating();
}

/// Asks for the PIN to retrieve the PID from an issuer that does not provide previews.
class WalletPersonalizeConfirmPinToRetrieve extends WalletPersonalizeState {
  const WalletPersonalizeConfirmPinToRetrieve();

  @override
  FlowProgress get stepperProgress =>
      FlowProgress(currentStep: OnboardingHelper.totalSteps - 2, totalSteps: OnboardingHelper.totalSteps);
}

class WalletPersonalizeCheckData extends WalletPersonalizeState {
  final List<Attribute> availableAttributes;
  @override
  final bool didGoBack;

  /// True when [availableAttributes] are of the issued PID instead of a preview, in which case the PID is only added
  /// to the wallet after the user accepts it and confirms with their PIN again.
  final bool issuedForReview;

  const WalletPersonalizeCheckData({
    required this.availableAttributes,
    this.didGoBack = false,
    this.issuedForReview = false,
  });

  @override
  FlowProgress get stepperProgress =>
      FlowProgress(currentStep: OnboardingHelper.totalSteps - 2, totalSteps: OnboardingHelper.totalSteps);

  @override
  List<Object?> get props => [availableAttributes, issuedForReview, ...super.props];
}

class WalletPersonalizeConfirmPin extends WalletPersonalizeState {
  /// Used to return to [WalletPersonalizeCheckData] when user presses back
  final List<Attribute> attributes;

  /// True when the PID was already issued for review, so confirming with the PIN commits it to the wallet.
  final bool issuedForReview;

  const WalletPersonalizeConfirmPin(this.attributes, {this.issuedForReview = false});

  @override
  FlowProgress get stepperProgress =>
      FlowProgress(currentStep: OnboardingHelper.totalSteps - 1, totalSteps: OnboardingHelper.totalSteps);

  @override
  bool get canGoBack => true;

  @override
  List<Object?> get props => [attributes, issuedForReview, ...super.props];
}

class WalletPersonalizeSuccess extends WalletPersonalizeState {
  final List<WalletCard> addedCards;
  final bool userCanTransfer;

  const WalletPersonalizeSuccess({required this.addedCards, required this.userCanTransfer});

  @override
  FlowProgress get stepperProgress =>
      FlowProgress(currentStep: OnboardingHelper.totalSteps, totalSteps: OnboardingHelper.totalSteps);

  @override
  List<Object?> get props => [addedCards, userCanTransfer, ...super.props];
}

class WalletPersonalizeDigidCancelled extends WalletPersonalizeState {
  const WalletPersonalizeDigidCancelled();
}

class WalletPersonalizeLoadInProgress extends WalletPersonalizeState {
  @override
  final FlowProgress? stepperProgress;

  const WalletPersonalizeLoadInProgress(this.stepperProgress);
}

class WalletPersonalizeAddingCards extends WalletPersonalizeState {
  @override
  final FlowProgress? stepperProgress;

  const WalletPersonalizeAddingCards(this.stepperProgress);
}

class WalletPersonalizeError extends WalletPersonalizeState implements ErrorState {
  @override
  final ApplicationError error;

  const WalletPersonalizeError({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}

/// Specific for errors during digid stage (non standard error ui)
class WalletPersonalizeDigidFailure extends WalletPersonalizeState implements ErrorState {
  @override
  final ApplicationError error;

  const WalletPersonalizeDigidFailure({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}
