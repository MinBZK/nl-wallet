part of 'issuance_bloc.dart';

sealed class IssuanceState extends Equatable {
  final bool isRefreshFlow;

  double get stepperProgress => 0.0;

  bool get showStopConfirmation => true;

  bool get canGoBack => false;

  bool get didGoBack => false;

  const IssuanceState({this.isRefreshFlow = false});

  @override
  List<Object?> get props => [
        isRefreshFlow,
        stepperProgress,
        showStopConfirmation,
        canGoBack,
        didGoBack,
      ];
}

class IssuanceInitial extends IssuanceState {
  const IssuanceInitial({super.isRefreshFlow});
}

class IssuanceLoadInProgress extends IssuanceState {
  const IssuanceLoadInProgress({super.isRefreshFlow});
}

class IssuanceLoadFailure extends IssuanceState {
  const IssuanceLoadFailure({super.isRefreshFlow});
}

class IssuanceCheckOrganization extends IssuanceState {
  final bool afterBackPressed;

  final Organization organization;

  const IssuanceCheckOrganization({
    required this.organization,
    this.afterBackPressed = false,
    super.isRefreshFlow,
  });

  @override
  double get stepperProgress => 0.2;

  @override
  bool get didGoBack => afterBackPressed;

  @override
  List<Object?> get props => [organization, ...super.props];
}

class IssuanceProofIdentity extends IssuanceState {
  final bool afterBackPressed;

  final Organization organization;

  final Policy policy;

  final List<Attribute> requestedAttributes;

  const IssuanceProofIdentity({
    super.isRefreshFlow,
    required this.organization,
    required this.policy,
    required this.requestedAttributes,
    this.afterBackPressed = false,
  });

  @override
  bool get canGoBack => !isRefreshFlow;

  @override
  bool get didGoBack => afterBackPressed;

  @override
  double get stepperProgress => 0.4;

  @override
  List<Object?> get props => [organization, policy, requestedAttributes, ...super.props];
}

class IssuanceMissingAttributes extends IssuanceState {
  final bool afterBackPressed;

  final Organization organization;

  final Policy policy;

  final List<MissingAttribute> missingAttributes;

  const IssuanceMissingAttributes({
    super.isRefreshFlow,
    required this.organization,
    required this.policy,
    required this.missingAttributes,
    this.afterBackPressed = false,
  });

  @override
  bool get canGoBack => !isRefreshFlow;

  @override
  bool get didGoBack => afterBackPressed;

  @override
  double get stepperProgress => 0.4;

  @override
  List<Object?> get props => [organization, policy, missingAttributes, ...super.props];
}

class IssuanceProvidePin extends IssuanceState {
  const IssuanceProvidePin({super.isRefreshFlow});

  @override
  bool get canGoBack => true;

  @override
  double get stepperProgress => 0.6;
}

class IssuanceCheckDataOffering extends IssuanceState {
  final WalletCard card;

  const IssuanceCheckDataOffering({super.isRefreshFlow, required this.card});

  @override
  double get stepperProgress => 0.8;

  @override
  List<Object?> get props => [card, ...super.props];
}

class IssuanceSelectCards extends IssuanceState {
  final MultipleCardsFlow multipleCardsFlow;

  final bool showNoSelectionError;

  @override
  final bool didGoBack;

  List<WalletCard> get cards => multipleCardsFlow.availableCards;

  List<WalletCard> get selectedCards => multipleCardsFlow.selectedCards;

  const IssuanceSelectCards({
    required this.multipleCardsFlow,
    this.didGoBack = false,
    this.showNoSelectionError = false,
    super.isRefreshFlow,
  });

  @override
  double get stepperProgress => 0.8;

  IssuanceSelectCards toggleCard(String cardId) {
    final selection = Set<String>.from(multipleCardsFlow.selectedCardIds);
    return IssuanceSelectCards(
      isRefreshFlow: isRefreshFlow,
      multipleCardsFlow: multipleCardsFlow.copyWith(selectedCardIds: selection..toggle(cardId)),
    );
  }

  IssuanceSelectCards copyWith({bool? showNoSelectionError}) {
    return IssuanceSelectCards(
      isRefreshFlow: isRefreshFlow,
      multipleCardsFlow: multipleCardsFlow,
      showNoSelectionError: showNoSelectionError ?? this.showNoSelectionError,
      didGoBack: didGoBack,
    );
  }

  @override
  List<Object?> get props => [cards, multipleCardsFlow, showNoSelectionError, ...super.props];
}

class IssuanceCheckCards extends IssuanceState {
  final MultipleCardsFlow multipleCardsFlow;

  @override
  final bool didGoBack;

  WalletCard get cardToCheck => multipleCardsFlow.activeCard;

  int get totalNrOfCardsToCheck => multipleCardsFlow.selectedCards.length;

  const IssuanceCheckCards({
    super.isRefreshFlow,
    required this.multipleCardsFlow,
    this.didGoBack = false,
  });

  IssuanceCheckCards copyForNextCard() {
    if (!multipleCardsFlow.hasMoreCards) throw UnsupportedError('There is no next card to check!');
    return IssuanceCheckCards(
      isRefreshFlow: isRefreshFlow,
      multipleCardsFlow: multipleCardsFlow.next(),
      didGoBack: false,
    );
  }

  IssuanceCheckCards copyForPreviousCard() {
    if (multipleCardsFlow.isAtFirstCard) throw UnsupportedError('There is no previous card to check!');
    return IssuanceCheckCards(
      isRefreshFlow: isRefreshFlow,
      multipleCardsFlow: multipleCardsFlow.previous(),
      didGoBack: true,
    );
  }

  @override
  double get stepperProgress => 0.9;

  @override
  bool get canGoBack => true;

  @override
  List<Object?> get props => [multipleCardsFlow, ...super.props];
}

class IssuanceCompleted extends IssuanceState {
  final List<WalletCard> addedCards;

  const IssuanceCompleted({super.isRefreshFlow, required this.addedCards});

  @override
  List<Object?> get props => [addedCards, ...super.props];

  @override
  bool get showStopConfirmation => false;
}

class IssuanceStopped extends IssuanceState {
  const IssuanceStopped({super.isRefreshFlow});

  @override
  bool get showStopConfirmation => false;
}

class IssuanceGenericError extends IssuanceState {
  const IssuanceGenericError({super.isRefreshFlow});

  @override
  bool get showStopConfirmation => false;
}

class IssuanceIdentityValidationFailure extends IssuanceState {
  const IssuanceIdentityValidationFailure({super.isRefreshFlow});

  @override
  bool get showStopConfirmation => false;
}
