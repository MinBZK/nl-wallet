part of 'wallet_personalize_bloc.dart';

const _kNrOfPages = 11;

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

  const WalletPersonalizeCheckData({required this.availableAttributes});

  @override
  double get stepperProgress => 7 / _kNrOfPages;

  @override
  List<Object?> get props => [availableAttributes, ...super.props];
}

class WalletPersonalizeScanIdIntro extends WalletPersonalizeState {
  final bool afterBackPressed;

  const WalletPersonalizeScanIdIntro({this.afterBackPressed = false});

  @override
  double get stepperProgress => 4 / _kNrOfPages;

  @override
  bool get didGoBack => afterBackPressed;
}

class WalletPersonalizeScanId extends WalletPersonalizeState {
  @override
  double get stepperProgress => 5 / _kNrOfPages;

  @override
  bool get canGoBack => true;
}

class WalletPersonalizeLoadingPhoto extends WalletPersonalizeState {
  final Duration mockedScanDuration;

  const WalletPersonalizeLoadingPhoto(this.mockedScanDuration);

  @override
  double get stepperProgress => 6 / _kNrOfPages;
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

class WalletPersonalizeRetrieveMoreCards extends WalletPersonalizeState {
  @override
  double get stepperProgress => 8 / _kNrOfPages;
}

class WalletPersonalizeSelectCards extends WalletPersonalizeState {
  final List<IssuanceResponse> issuanceResponses;
  final List<String> selectedCardIds;

  List<WalletCard> get availableCards => issuanceResponses.map((e) => e.cards).flattened.toList();

  List<WalletCard> get selectedCards => availableCards.where((card) => selectedCardIds.contains(card.id)).toList();

  const WalletPersonalizeSelectCards({
    required this.issuanceResponses,
    required this.selectedCardIds,
  });

  @override
  double get stepperProgress => 9 / _kNrOfPages;

  @override
  List<Object?> get props => [issuanceResponses, selectedCardIds, ...super.props];
}

class WalletPersonalizeLoadInProgress extends WalletPersonalizeState {
  final double step;

  const WalletPersonalizeLoadInProgress(this.step);

  @override
  double get stepperProgress => step / _kNrOfPages;

  @override
  List<Object?> get props => [step, ...super.props];
}
