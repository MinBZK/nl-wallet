class CardDataScreenArgument {
  static const _kCardIdKey = 'cardId';
  static const _kCardTitleKey = 'cardTitle';

  final String cardId;
  final String cardTitle;

  const CardDataScreenArgument({required this.cardId, required this.cardTitle});

  Map<String, dynamic> toMap() {
    return {
      _kCardIdKey: cardId,
      _kCardTitleKey: cardTitle,
    };
  }

  static CardDataScreenArgument fromMap(Map<String, dynamic> map) {
    return CardDataScreenArgument(
      cardId: map[_kCardIdKey],
      cardTitle: map[_kCardTitleKey],
    );
  }
}
