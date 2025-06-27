part of 'issuance_bloc.dart';

const int kIssuanceSteps = 5;

sealed class IssuanceState extends Equatable {
  FlowProgress get stepperProgress => const FlowProgress(currentStep: kIssuanceSteps, totalSteps: kIssuanceSteps);

  bool get showStopConfirmation => true;

  bool get canGoBack => false;

  bool get didGoBack => false;

  const IssuanceState();

  @override
  List<Object?> get props => [
        stepperProgress,
        showStopConfirmation,
        canGoBack,
        didGoBack,
      ];
}

class IssuanceInitial extends IssuanceState {
  const IssuanceInitial();
}

class IssuanceLoadInProgress extends IssuanceState {
  @override
  final FlowProgress stepperProgress;

  const IssuanceLoadInProgress(this.stepperProgress);
}

class IssuanceCheckOrganization extends IssuanceState {
  final bool afterBackPressed;

  final Organization organization;
  final Policy policy;
  final List<DiscloseCardRequest> cardRequests;
  final LocalizedText purpose;

  const IssuanceCheckOrganization({
    required this.organization,
    required this.cardRequests,
    required this.policy,
    required this.purpose,
    this.afterBackPressed = false,
  });

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 1, totalSteps: kIssuanceSteps);

  @override
  bool get didGoBack => afterBackPressed;

  @override
  List<Object?> get props => [organization, policy, cardRequests, purpose, ...super.props];

  /// Returns a new [IssuanceCheckOrganization] with the updated request
  IssuanceCheckOrganization updateWith(DiscloseCardRequest updatedEntry) {
    final updatedCardRequests = cardRequests.replace(updatedEntry, (it) => it.id);
    return copyWith(cardRequests: updatedCardRequests);
  }

  /// Creates a new [IssuanceCheckOrganization] instance with the same properties as this one,
  /// but with the provided properties updated.
  IssuanceCheckOrganization copyWith({
    Organization? organization,
    List<DiscloseCardRequest>? cardRequests,
    Policy? policy,
    LocalizedText? purpose,
    bool? afterBackPressed,
  }) {
    return IssuanceCheckOrganization(
      organization: organization ?? this.organization,
      cardRequests: cardRequests ?? this.cardRequests,
      policy: policy ?? this.policy,
      purpose: purpose ?? this.purpose,
      afterBackPressed: afterBackPressed ?? this.afterBackPressed,
    );
  }
}

class IssuanceMissingAttributes extends IssuanceState {
  final bool afterBackPressed;

  final Organization organization;

  final List<MissingAttribute> missingAttributes;

  const IssuanceMissingAttributes({
    required this.organization,
    required this.missingAttributes,
    this.afterBackPressed = false,
  });

  @override
  bool get didGoBack => afterBackPressed;

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: kIssuanceSteps, totalSteps: kIssuanceSteps);

  @override
  List<Object?> get props => [organization, missingAttributes, ...super.props];
}

class IssuanceProvidePinForDisclosure extends IssuanceState {
  const IssuanceProvidePinForDisclosure();

  @override
  bool get canGoBack => true;

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 2, totalSteps: kIssuanceSteps);
}

class IssuanceReviewCards extends IssuanceState {
  /// A map of all cards available for selection, where the key is the [WalletCard]
  /// and the value represents whether the card is selected.
  final Map<WalletCard, bool /* selected */ > selectableCards;

  /// Returns the list of cards that are not persisted (i.e., newly created cards).
  List<WalletCard> get offeredCards => selectableCards.keys.where((card) => !card.isPersisted).toList();

  /// Returns the list of persisted cards (i.e., updated cards).
  List<WalletCard> get renewedCards => selectableCards.keys.where((card) => card.isPersisted).toList();

  /// Returns the list of currently selected cards based on the [selectableCards] map.
  /// Includes both newly created and updated cards that are selected.
  List<WalletCard> get selectedCards =>
      selectableCards.entries.where((entry) => entry.value).map((entry) => entry.key).toList();

  final bool afterBackPressed;

  @override
  bool get didGoBack => afterBackPressed;

  const IssuanceReviewCards({required this.selectableCards, this.afterBackPressed = false});

  /// Create a IssuanceReviewCards state where all provided cards default to being selected
  factory IssuanceReviewCards.init({required List<WalletCard> cards, bool afterBackPressed = false}) {
    final selectableCards = cards.asMap().map((_, card) => MapEntry(card, true));
    return IssuanceReviewCards(selectableCards: selectableCards, afterBackPressed: afterBackPressed);
  }

  /// Create an IssuanceReviewCards state for which the provided card's selected state is toggled
  IssuanceReviewCards toggleCard(WalletCard card) {
    assert(selectableCards.containsKey(card), 'Can not toggle card that does not exist');
    final updatedSelection = Map<WalletCard, bool>.from(selectableCards)..update(card, (selected) => !selected);
    return IssuanceReviewCards(selectableCards: updatedSelection);
  }

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 3, totalSteps: kIssuanceSteps);

  @override
  List<Object?> get props => [selectableCards, ...super.props];
}

class IssuanceProvidePinForIssuance extends IssuanceState {
  final List<WalletCard> cards;

  const IssuanceProvidePinForIssuance({required this.cards});

  @override
  bool get canGoBack => true;

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: 4, totalSteps: kIssuanceSteps);

  @override
  List<Object?> get props => [cards, ...super.props];
}

class IssuanceCompleted extends IssuanceState {
  final List<WalletCard> addedCards;

  const IssuanceCompleted({required this.addedCards});

  @override
  FlowProgress get stepperProgress => const FlowProgress(currentStep: kIssuanceSteps, totalSteps: kIssuanceSteps);

  @override
  List<Object?> get props => [addedCards, ...super.props];

  @override
  bool get showStopConfirmation => false;
}

class IssuanceStopped extends IssuanceState {
  final String? returnUrl;

  const IssuanceStopped({this.returnUrl});

  @override
  bool get showStopConfirmation => false;

  @override
  List<Object?> get props => [returnUrl, ...super.props];
}

class IssuanceGenericError extends IssuanceState implements ErrorState {
  @override
  final ApplicationError error;

  final String? returnUrl;

  const IssuanceGenericError({required this.error, this.returnUrl});

  @override
  bool get showStopConfirmation => false;

  @override
  List<Object?> get props => [...super.props, error, returnUrl];
}

class IssuanceExternalScannerError extends IssuanceState implements ErrorState {
  @override
  final ApplicationError error;

  const IssuanceExternalScannerError({required this.error});

  @override
  bool get showStopConfirmation => false;

  @override
  List<Object?> get props => [...super.props, error];
}

class IssuanceNoCardsRetrieved extends IssuanceState {
  final Organization organization;

  const IssuanceNoCardsRetrieved({required this.organization});

  @override
  bool get showStopConfirmation => false;

  @override
  List<Object?> get props => [organization, ...super.props];
}

class IssuanceNetworkError extends IssuanceState implements NetworkErrorState {
  @override
  final bool hasInternet;

  @override
  final ApplicationError error;

  @override
  final int? statusCode;

  const IssuanceNetworkError({required this.error, required this.hasInternet, this.statusCode});

  @override
  bool get showStopConfirmation => false;

  @override
  List<Object?> get props => [...super.props, error, hasInternet, statusCode];
}

class IssuanceSessionExpired extends IssuanceState implements ErrorState {
  @override
  final ApplicationError error;

  @override
  bool get showStopConfirmation => false;

  final bool isCrossDevice;

  final bool canRetry;

  final String? returnUrl;

  const IssuanceSessionExpired({
    required this.error,
    required this.isCrossDevice,
    required this.canRetry,
    this.returnUrl,
  });

  @override
  List<Object?> get props => [error, canRetry, isCrossDevice, returnUrl, ...super.props];
}

/// State that is exposed when the session has been stopped remotely (e.g. the user pressed stop in wallet_web)
class IssuanceSessionCancelled extends IssuanceState implements ErrorState {
  final Organization? relyingParty;
  final String? returnUrl;

  @override
  final ApplicationError error;

  @override
  bool get showStopConfirmation => false;

  const IssuanceSessionCancelled({
    required this.error,
    this.relyingParty,
    this.returnUrl,
  });

  @override
  List<Object?> get props => [error, relyingParty, returnUrl, ...super.props];
}

class IssuanceRelyingPartyError extends IssuanceState implements ErrorState {
  @override
  final ApplicationError error;

  final LocalizedText? organizationName;

  @override
  bool get showStopConfirmation => false;

  const IssuanceRelyingPartyError({required this.error, this.organizationName});

  @override
  List<Object?> get props => [error, organizationName, ...super.props];
}
