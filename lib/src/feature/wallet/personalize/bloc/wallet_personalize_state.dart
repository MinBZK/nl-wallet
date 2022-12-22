part of 'wallet_personalize_bloc.dart';

const _kNrOfPages = 13;

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
  final MultipleCardsFlow multipleCardsFlow;

  @override
  final bool didGoBack;

  List<WalletCard> get availableCards => multipleCardsFlow.availableCards;

  List<WalletCard> get selectedCards => multipleCardsFlow.selectedCards;

  const WalletPersonalizeSelectCards({
    required this.multipleCardsFlow,
    this.didGoBack = false,
  });

  @override
  double get stepperProgress => 9 / _kNrOfPages;

  @override
  List<Object?> get props => [multipleCardsFlow, ...super.props];

  WalletPersonalizeSelectCards toggleCard(String cardId) {
    final selection = Set<String>.from(multipleCardsFlow.selectedCardIds);
    return WalletPersonalizeSelectCards(
      multipleCardsFlow: multipleCardsFlow.copyWith(selectedCardIds: selection..toggle(cardId)),
    );
  }
}

class WalletPersonalizeCheckCards extends WalletPersonalizeState {
  final MultipleCardsFlow multipleCardsFlow;

  @override
  final bool didGoBack;

  List<WalletCard> get availableCards => multipleCardsFlow.cardToOrganizations.keys.toList();

  List<WalletCard> get selectedCards => multipleCardsFlow.selectedCards;

  WalletCard get cardToCheck => multipleCardsFlow.activeCard;

  int get totalNrOfCardsToCheck => multipleCardsFlow.selectedCardIds.length;

  bool get hasMoreCards => multipleCardsFlow.hasMoreCards;

  const WalletPersonalizeCheckCards({
    required this.multipleCardsFlow,
    this.didGoBack = false,
  });

  WalletPersonalizeCheckCards copyForNextCard() {
    if (!multipleCardsFlow.hasMoreCards) throw UnsupportedError('There is no next card to check!');
    return WalletPersonalizeCheckCards(multipleCardsFlow: multipleCardsFlow.next());
  }

  WalletPersonalizeCheckCards copyForPreviousCard() {
    if (multipleCardsFlow.isAtFirstCard) throw UnsupportedError('There is no previous card to check!');
    return WalletPersonalizeCheckCards(multipleCardsFlow: multipleCardsFlow.previous(), didGoBack: true);
  }

  @override
  double get stepperProgress {
    if (totalNrOfCardsToCheck <= 1) return 10 / _kNrOfPages;
    final checkCardsProgress = (multipleCardsFlow.activeIndex / (totalNrOfCardsToCheck - 1));
    return (10 + checkCardsProgress) / _kNrOfPages;
  }

  @override
  List<Object?> get props => [multipleCardsFlow, ...super.props];

  @override
  bool get canGoBack => true;
}

class WalletPersonalizeConfirmPin extends WalletPersonalizeState {
  final MultipleCardsFlow multipleCardsFlow;

  List<WalletCard> get availableCards => multipleCardsFlow.availableCards;

  List<WalletCard> get selectedCards => multipleCardsFlow.selectedCards;

  const WalletPersonalizeConfirmPin({
    required this.multipleCardsFlow,
  });

  @override
  double get stepperProgress => 12 / _kNrOfPages;

  @override
  bool get canGoBack => true;

  @override
  List<Object?> get props => [multipleCardsFlow, ...super.props];
}

class WalletPersonalizeLoadInProgress extends WalletPersonalizeState {
  final double step;

  const WalletPersonalizeLoadInProgress(this.step);

  @override
  double get stepperProgress => step / _kNrOfPages;

  @override
  List<Object?> get props => [step, ...super.props];
}
