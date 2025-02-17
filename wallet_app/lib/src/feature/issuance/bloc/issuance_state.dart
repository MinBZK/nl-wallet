part of 'issuance_bloc.dart';

const int kIssuanceSteps = 7;

sealed class IssuanceState extends Equatable {
  final bool isRefreshFlow;

  FlowProgress get stepperProgress => const FlowProgress(currentStep: 0, totalSteps: kIssuanceSteps);

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
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 1, totalSteps: kIssuanceSteps);

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
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 2, totalSteps: kIssuanceSteps);

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
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 2, totalSteps: kIssuanceSteps);

  @override
  List<Object?> get props => [organization, policy, missingAttributes, ...super.props];
}

class IssuanceProvidePin extends IssuanceState {
  const IssuanceProvidePin({super.isRefreshFlow});

  @override
  bool get canGoBack => true;

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 4, totalSteps: kIssuanceSteps);
}

class IssuanceCheckDataOffering extends IssuanceState {
  final WalletCard card;

  const IssuanceCheckDataOffering({super.isRefreshFlow, required this.card});

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 5, totalSteps: kIssuanceSteps);

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
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 5, totalSteps: kIssuanceSteps);

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
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 6, totalSteps: kIssuanceSteps);

  @override
  bool get canGoBack => true;

  @override
  List<Object?> get props => [multipleCardsFlow, ...super.props];
}

class IssuanceCompleted extends IssuanceState {
  final List<WalletCard> addedCards;

  const IssuanceCompleted({super.isRefreshFlow, required this.addedCards});

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 7, totalSteps: kIssuanceSteps);

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

class IssuanceGenericError extends IssuanceState implements ErrorState {
  @override
  final ApplicationError error;

  const IssuanceGenericError({required this.error, super.isRefreshFlow});

  @override
  bool get showStopConfirmation => false;

  @override
  List<Object?> get props => [...super.props, error];
}

class IssuanceIdentityValidationFailure extends IssuanceState {
  const IssuanceIdentityValidationFailure({super.isRefreshFlow});

  @override
  bool get showStopConfirmation => false;
}
