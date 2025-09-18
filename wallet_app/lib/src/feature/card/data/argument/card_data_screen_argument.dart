import 'package:flutter/foundation.dart';

@immutable
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

  CardDataScreenArgument.fromMap(Map<String, dynamic> map) : cardId = map[_kCardIdKey], cardTitle = map[_kCardTitleKey];

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is CardDataScreenArgument &&
          runtimeType == other.runtimeType &&
          cardId == other.cardId &&
          cardTitle == other.cardTitle;

  @override
  int get hashCode => Object.hash(
    runtimeType,
    cardId,
    cardTitle,
  );
}
