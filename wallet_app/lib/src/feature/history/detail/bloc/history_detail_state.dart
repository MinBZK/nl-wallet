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
  final TimelineAttribute timelineAttribute;

  static bool _verifyAllRelatedCardsProvided(List<WalletCard> cards, List<DataAttribute> dataAttributes) {
    final availableCardIds = cards.map((e) => e.id).toSet().sorted();
    final requiredCardIds = dataAttributes.map((e) => e.sourceCardId).toSet().sorted();
    return const DeepCollectionEquality().equals(availableCardIds, requiredCardIds);
  }

  HistoryDetailLoadSuccess(this.timelineAttribute, this.relatedCards)
      : assert(_verifyAllRelatedCardsProvided(relatedCards, timelineAttribute.dataAttributes));

  /// Groups the [DataAttribute]s with the [WalletCard] they are sourced from.
  /// The call to [cardById] is safely force unwrapped because we assert [_verifyAllRelatedCardsProvided]
  /// when an instance of [HistoryDetailLoadSuccess] is created.
  Map<WalletCard, List<DataAttribute>> get attributesByCard =>
      timelineAttribute.attributesByCardId.map((key, value) => MapEntry(cardById(key)!, value));

  WalletCard? cardById(String cardId) => relatedCards.firstWhereOrNull((card) => cardId == card.id);

  @override
  List<Object> get props => [relatedCards, timelineAttribute];
}

class HistoryDetailLoadFailure extends HistoryDetailState {
  const HistoryDetailLoadFailure();

  @override
  List<Object> get props => [];
}
