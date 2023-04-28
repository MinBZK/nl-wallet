class CardSummaryScreenArgument {
  static const _kCardIdKey = 'cardId';
  static const _kCardTitleKey = 'cardTitle';

  final String cardId;
  final String cardTitle;

  const CardSummaryScreenArgument({required this.cardId, required this.cardTitle});

  Map<String, dynamic> toMap() {
    return {
      _kCardIdKey: cardId,
      _kCardTitleKey: cardTitle,
    };
  }

  static CardSummaryScreenArgument fromMap(Map<String, dynamic> map) {
    return CardSummaryScreenArgument(
      cardId: map[_kCardIdKey],
      cardTitle: map[_kCardTitleKey],
    );
  }
}
