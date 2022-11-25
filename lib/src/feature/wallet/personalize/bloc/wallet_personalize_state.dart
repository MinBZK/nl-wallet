part of 'wallet_personalize_bloc.dart';

const _kNrOfPages = 4;

abstract class WalletPersonalizeState extends Equatable {
  double get stepperProgress => 0.0;

  bool get didGoBack => false;

  const WalletPersonalizeState();

  @override
  List<Object?> get props => [stepperProgress, didGoBack];
}

class WalletPersonalizeInitial extends WalletPersonalizeState {
  @override
  double get stepperProgress => 1 / _kNrOfPages;
}

class WalletPersonalizeLoadingPid extends WalletPersonalizeState {
  @override
  double get stepperProgress => 2 / _kNrOfPages;
}

class WalletPersonalizeCheckData extends WalletPersonalizeState {
  final WalletCard pidCard;

  const WalletPersonalizeCheckData(this.pidCard);

  @override
  double get stepperProgress => 3 / _kNrOfPages;

  @override
  List<Object?> get props => [pidCard, ...super.props];
}

class WalletPersonalizeSuccess extends WalletPersonalizeState {
  final WalletCard pidCard;

  const WalletPersonalizeSuccess(this.pidCard);

  CardFront get cardFront => pidCard.front;

  @override
  double get stepperProgress => 4 / _kNrOfPages;
}

class WalletPersonalizeFailure extends WalletPersonalizeState {
  @override
  double get stepperProgress => 0 / _kNrOfPages;
}
