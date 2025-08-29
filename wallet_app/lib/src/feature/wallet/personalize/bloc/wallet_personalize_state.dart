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
      FlowProgress(currentStep: SetupHelper.totalSetupSteps - 3, totalSteps: SetupHelper.totalSetupSteps);
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

class WalletPersonalizeCheckData extends WalletPersonalizeState {
  final List<Attribute> availableAttributes;
  @override
  final bool didGoBack;

  const WalletPersonalizeCheckData({required this.availableAttributes, this.didGoBack = false});

  @override
  FlowProgress get stepperProgress =>
      FlowProgress(currentStep: SetupHelper.totalSetupSteps - 2, totalSteps: SetupHelper.totalSetupSteps);

  @override
  List<Object?> get props => [availableAttributes, ...super.props];
}

class WalletPersonalizeConfirmPin extends WalletPersonalizeState {
  /// Used to return to [WalletPersonalizeCheckData] when user presses back
  final List<Attribute> attributes;

  const WalletPersonalizeConfirmPin(this.attributes);

  @override
  FlowProgress get stepperProgress =>
      FlowProgress(currentStep: SetupHelper.totalSetupSteps - 1, totalSteps: SetupHelper.totalSetupSteps);

  @override
  bool get canGoBack => true;

  @override
  List<Object?> get props => [attributes, ...super.props];
}

class WalletPersonalizeSuccess extends WalletPersonalizeState {
  final List<WalletCard> addedCards;

  const WalletPersonalizeSuccess(this.addedCards);

  @override
  FlowProgress get stepperProgress =>
      FlowProgress(currentStep: SetupHelper.totalSetupSteps, totalSteps: SetupHelper.totalSetupSteps);

  @override
  List<Object?> get props => [addedCards, ...super.props];
}

class WalletPersonalizeFailure extends WalletPersonalizeState {}

class WalletPersonalizeDigidFailure extends WalletPersonalizeState implements ErrorState {
  @override
  final ApplicationError error;

  const WalletPersonalizeDigidFailure({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}

class WalletPersonalizeDigidCancelled extends WalletPersonalizeState {}

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

class WalletPersonalizeNetworkError extends WalletPersonalizeState implements NetworkErrorState {
  @override
  final ApplicationError error;

  @override
  final bool hasInternet;

  @override
  final int? statusCode;

  const WalletPersonalizeNetworkError({required this.error, required this.hasInternet, this.statusCode});

  @override
  List<Object?> get props => [error, hasInternet, statusCode, ...super.props];
}

class WalletPersonalizeGenericError extends WalletPersonalizeState implements ErrorState {
  @override
  final ApplicationError error;

  const WalletPersonalizeGenericError({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}

class WalletPersonalizeSessionExpired extends WalletPersonalizeState implements ErrorState {
  @override
  final ApplicationError error;

  const WalletPersonalizeSessionExpired({required this.error});

  @override
  List<Object?> get props => [error, ...super.props];
}

class WalletPersonalizeRelyingPartyError extends WalletPersonalizeState implements ErrorState {
  @override
  final ApplicationError error;

  final LocalizedText? organizationName;

  const WalletPersonalizeRelyingPartyError({required this.error, this.organizationName});

  @override
  List<Object?> get props => [error, organizationName, ...super.props];
}
