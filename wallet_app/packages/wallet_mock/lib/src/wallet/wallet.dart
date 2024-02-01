import 'package:collection/collection.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet_core/core.dart';

import '../data/mock/mock_organizations.dart';

class Wallet {
  final BehaviorSubject<bool> _isLockedSubject = BehaviorSubject.seeded(true);
  final BehaviorSubject<List<Card>> _cardsSubject = BehaviorSubject.seeded([]);

  Wallet();

  Stream<bool> get lockedStream => _isLockedSubject.stream;

  Stream<List<Card>> get cardsStream => _cardsSubject.stream;

  List<Card> get _cards => _cardsSubject.value;

  List<CardAttribute> get _allAttributes => _cards.map((card) => card.attributes).flattened.toList();

  bool get isEmpty => _cards.isEmpty;

  bool containsAttributes(Iterable<String> keys) => keys.every((key) => containsAttribute(key));

  bool containsAttribute(String attributeKey) {
    return _cards.any((element) => element.attributes.any((element) => element.key == attributeKey));
  }

  CardAttribute? findAttribute(String key) => _allAttributes.firstWhereOrNull((attribute) => attribute.key == key);

  List<DisclosureCard> getDisclosureCards(Iterable<String> keys) {
    final allRequestedAttributes = keys.map((key) => findAttribute(key)).nonNulls;
    final cardToAttributes = allRequestedAttributes
        .groupListsBy((attribute) => _cards.firstWhere((card) => card.attributes.contains(attribute)));
    return cardToAttributes.entries
        .map(
          (e) => DisclosureCard(
            docType: e.key.docType,
            attributes: e.value,
            issuer: kOrganizations[kRvigId]!,
          ),
        )
        .toList();
  }

  List<String> getMissingAttributeKeys(Iterable<String> keys) {
    final allAvailableKeys = _allAttributes.map((attribute) => attribute.key);
    final requestedAttributesSet = keys.toSet();
    requestedAttributesSet.removeAll(allAvailableKeys);
    return requestedAttributesSet.toList();
  }

  void reset() {
    _cardsSubject.add([]);
    _isLockedSubject.add(true);
  }

  lock() => _isLockedSubject.add(true);

  unlock() => _isLockedSubject.add(false);

  /// Adds the cards to the wallet, if a card with the same docType already exists, it is replaced with the new card.
  void add(List<Card> cards) {
    final currentCards = List.of(_cards);
    final newDocTypes = cards.map((e) => e.docType);
    final cardsToKeep = currentCards.whereNot((card) => newDocTypes.contains(card.docType));
    final newCardList = List.of(cardsToKeep)..addAll(cards);
    _cardsSubject.add(newCardList);
  }
}
