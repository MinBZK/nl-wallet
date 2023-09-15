part of 'wallet_personalize_bloc.dart';

const _kNrOfPages = 6;

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
  double get stepperProgress => 1 / _kNrOfPages;
}

class WalletPersonalizeLoadingIssuanceUrl extends WalletPersonalizeState {
  const WalletPersonalizeLoadingIssuanceUrl();

  @override
  double get stepperProgress => 2 / _kNrOfPages;

  @override
  List<Object?> get props => [...super.props];
}

class WalletPersonalizeConnectDigid extends WalletPersonalizeState {
  final String authUrl;

  const WalletPersonalizeConnectDigid(this.authUrl);

  @override
  double get stepperProgress => 2 / _kNrOfPages;

  @override
  List<Object?> get props => [...super.props, authUrl];
}

class WalletPersonalizeAuthenticating extends WalletPersonalizeState {
  const WalletPersonalizeAuthenticating();

  @override
  double get stepperProgress => 2 / _kNrOfPages;
}

class WalletPersonalizeCheckData extends WalletPersonalizeState {
  final List<Attribute> availableAttributes;
  @override
  final bool didGoBack;

  const WalletPersonalizeCheckData({required this.availableAttributes, this.didGoBack = false});

  @override
  double get stepperProgress => 3 / _kNrOfPages;

  @override
  List<Object?> get props => [availableAttributes, ...super.props];
}

class WalletPersonalizeConfirmPin extends WalletPersonalizeState {
  /// Used to return to [WalletPersonalizeCheckData] when user presses back
  final List<Attribute> attributes;

  const WalletPersonalizeConfirmPin(this.attributes);

  @override
  double get stepperProgress => 4 / _kNrOfPages;

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
  double get stepperProgress => 1;
}

class WalletPersonalizeDigidFailure extends WalletPersonalizeState {
  @override
  double get stepperProgress => 1;
}

class WalletPersonalizeDigidCancelled extends WalletPersonalizeState {
  @override
  double get stepperProgress => 1;
}

class WalletPersonalizeLoadInProgress extends WalletPersonalizeState {
  final double step;

  const WalletPersonalizeLoadInProgress(this.step);

  @override
  double get stepperProgress => step / _kNrOfPages;

  @override
  List<Object?> get props => [step, ...super.props];
}
