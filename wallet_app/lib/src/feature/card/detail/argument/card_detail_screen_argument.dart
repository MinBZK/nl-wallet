import '../../../../domain/model/wallet_card.dart';

class CardDetailScreenArgument {
  static const _kCardKey = 'card';
  static const _kCardIdKey = 'cardId';
  static const _kCardTitleKey = 'cardTitle';

  final WalletCard? card;
  final String cardId;
  final String cardTitle;

  const CardDetailScreenArgument({this.card, required this.cardId, required this.cardTitle});

  factory CardDetailScreenArgument.forCard(WalletCard card) => CardDetailScreenArgument(
        card: card,
        cardId: card.id,
        cardTitle: card.front.title,
      );

  Map<String, dynamic> toMap() {
    return {
      _kCardKey: card?.toJson() ?? '',
      _kCardIdKey: cardId,
      _kCardTitleKey: cardTitle,
    };
  }

  static CardDetailScreenArgument fromMap(Map<String, dynamic> map) {
    return CardDetailScreenArgument(
      card: map[_kCardKey].isEmpty ? null : WalletCard.fromJson(map[_kCardKey]),
      cardId: map[_kCardIdKey],
      cardTitle: map[_kCardTitleKey],
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is CardDetailScreenArgument &&
          runtimeType == other.runtimeType &&
          card == other.card &&
          cardId == other.cardId &&
          cardTitle == other.cardTitle;

  @override
  int get hashCode => Object.hash(
        runtimeType,
        card,
        cardId,
        cardTitle,
      );
}
