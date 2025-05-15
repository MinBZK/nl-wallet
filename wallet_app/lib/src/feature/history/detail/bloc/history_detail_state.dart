part of 'history_detail_bloc.dart';

sealed class HistoryDetailState extends Equatable {
  const HistoryDetailState();
}

class HistoryDetailInitial extends HistoryDetailState {
  @override
  List<Object> get props => [];
}

class HistoryDetailLoadInProgress extends HistoryDetailState {
  const HistoryDetailLoadInProgress();

  @override
  List<Object?> get props => [];
}

class HistoryDetailLoadSuccess extends HistoryDetailState {
  final List<WalletCard> relatedCards;
  final WalletEvent event;

  static bool _verifyAllRelatedCardsProvided(List<WalletCard> cards, List<DataAttribute> dataAttributes) {
    final availableCardIds = cards.map((card) => card.docType).toSet();
    final requiredCardIds = dataAttributes.map((e) => e.sourceCardDocType).toSet();
    return availableCardIds.containsAll(requiredCardIds);
  }

  HistoryDetailLoadSuccess(this.event, this.relatedCards)
      : assert(
          _verifyAllRelatedCardsProvided(relatedCards, event.sharedAttributes),
          'All cards of which data is provided should also be available',
        );

  /// Groups the [DataAttribute]s with the [WalletCard] they are sourced from.
  /// The call to [cardByDocType] is safely force unwrapped because we assert [_verifyAllRelatedCardsProvided]
  /// when an instance of [HistoryDetailLoadSuccess] is created.
  RequestedAttributes get attributesByCard =>
      event.attributesByDocType.map((key, value) => MapEntry(cardByDocType(key)!, value));

  WalletCard? cardByDocType(String docType) => relatedCards.firstWhereOrNull((card) => docType == card.docType);

  @override
  List<Object> get props => [relatedCards, event];
}

class HistoryDetailLoadFailure extends HistoryDetailState {
  const HistoryDetailLoadFailure();

  @override
  List<Object> get props => [];
}
