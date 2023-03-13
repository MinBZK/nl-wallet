part of 'wallet_personalize_bloc.dart';

const _kNrOfPages = 6;

abstract class WalletPersonalizeState extends Equatable {
  double get stepperProgress => 0.0;

  bool get canGoBack => false;

  bool get didGoBack => false;

  const WalletPersonalizeState();

  @override
  List<Object?> get props => [stepperProgress, didGoBack, canGoBack];
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
  final List<DataAttribute> availableAttributes;
  @override
  final bool didGoBack;

  const WalletPersonalizeCheckData({required this.availableAttributes, this.didGoBack = false});

  @override
  double get stepperProgress => 3 / _kNrOfPages;

  @override
  List<Object?> get props => [availableAttributes, ...super.props];
}

class WalletPersonalizeConfirmPin extends WalletPersonalizeState {
  const WalletPersonalizeConfirmPin();

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

  List<CardFront> get cardFronts => addedCards.map((e) => e.front).toList();

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
