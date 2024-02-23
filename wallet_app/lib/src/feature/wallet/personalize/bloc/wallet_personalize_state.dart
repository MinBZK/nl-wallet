part of 'wallet_personalize_bloc.dart';

sealed class WalletPersonalizeState extends Equatable {
  double get stepperProgress => 0.0;

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
  double get stepperProgress => 0.56;
}

class WalletPersonalizeLoadingIssuanceUrl extends WalletPersonalizeState {
  const WalletPersonalizeLoadingIssuanceUrl();

  @override
  double get stepperProgress => 0.64;

  @override
  List<Object?> get props => [...super.props];
}

class WalletPersonalizeConnectDigid extends WalletPersonalizeState {
  final String authUrl;

  const WalletPersonalizeConnectDigid(this.authUrl);

  @override
  double get stepperProgress => 0.72;

  @override
  List<Object?> get props => [...super.props, authUrl];
}

class WalletPersonalizeAuthenticating extends WalletPersonalizeState {
  const WalletPersonalizeAuthenticating();

  @override
  double get stepperProgress => 0.72;
}

class WalletPersonalizeCheckData extends WalletPersonalizeState {
  final List<Attribute> availableAttributes;
  @override
  final bool didGoBack;

  const WalletPersonalizeCheckData({required this.availableAttributes, this.didGoBack = false});

  @override
  double get stepperProgress => 0.8;

  @override
  List<Object?> get props => [availableAttributes, ...super.props];
}

class WalletPersonalizeConfirmPin extends WalletPersonalizeState {
  /// Used to return to [WalletPersonalizeCheckData] when user presses back
  final List<Attribute> attributes;

  const WalletPersonalizeConfirmPin(this.attributes);

  @override
  double get stepperProgress => 0.88;

  @override
  bool get canGoBack => true;

  @override
  List<Object?> get props => [...super.props];
}

class WalletPersonalizeSuccess extends WalletPersonalizeState {
  final List<WalletCard> addedCards;

  const WalletPersonalizeSuccess(this.addedCards);

  @override
  double get stepperProgress => 1;

  @override
  List<Object?> get props => [addedCards, ...super.props];
}

class WalletPersonalizeFailure extends WalletPersonalizeState {
  @override
  double get stepperProgress => 0;
}

class WalletPersonalizeDigidFailure extends WalletPersonalizeState {
  @override
  double get stepperProgress => 0;
}

class WalletPersonalizeDigidCancelled extends WalletPersonalizeState {
  @override
  double get stepperProgress => 0;
}

class WalletPersonalizeLoadInProgress extends WalletPersonalizeState {
  final double progress;

  const WalletPersonalizeLoadInProgress(this.progress);

  @override
  double get stepperProgress => progress;

  @override
  List<Object?> get props => [progress, ...super.props];
}

class WalletPersonalizeNetworkError extends WalletPersonalizeState implements NetworkError {
  @override
  final int? statusCode;

  @override
  final bool hasInternet;

  const WalletPersonalizeNetworkError({required this.hasInternet, this.statusCode});

  @override
  List<Object?> get props => [hasInternet, statusCode, ...super.props];
}
