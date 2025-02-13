import 'package:collection/collection.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet_core/core.dart';

import '../data/mock/mock_organizations.dart';

class Wallet {
  final BehaviorSubject<bool> _isLockedSubject = BehaviorSubject.seeded(true);
  final BehaviorSubject<List<Attestation>> _attestationsSubject = BehaviorSubject.seeded([]);

  Wallet();

  Stream<bool> get lockedStream => _isLockedSubject.stream;

  Stream<List<Attestation>> get attestationsStream => _attestationsSubject.stream;

  List<Attestation> get _attestations => _attestationsSubject.value;

  List<AttestationAttribute> get _allAttributes =>
      _attestations.map((attestation) => attestation.attributes).flattened.toList();

  bool get isEmpty => _attestations.isEmpty;

  bool containsAttributes(Iterable<String> keys) => keys.every(containsAttribute);

  bool containsAttribute(String attributeKey) {
    return _attestations.any((element) => element.attributes.any((element) => element.key == attributeKey));
  }

  AttestationAttribute? findAttribute(String key) =>
      _allAttributes.firstWhereOrNull((attribute) => attribute.key == key);

  List<DisclosureCard> getDisclosureCards(Iterable<String> keys) {
    final allRequestedAttributes = keys.map(findAttribute).nonNulls;
    final cardToAttributes = allRequestedAttributes
        .groupListsBy((attribute) => _attestations.firstWhere((card) => card.attributes.contains(attribute)));
    return cardToAttributes.entries
        .map(
          (e) => DisclosureCard(
            docType: e.key.attestationType,
            attributes: e.value,
            displayMetadata: e.key.displayMetadata,
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
    _attestationsSubject.add([]);
    _isLockedSubject.add(true);
  }

  lock() => _isLockedSubject.add(true);

  unlock() => _isLockedSubject.add(false);

  /// Adds the cards to the wallet, if a card with the same docType already exists, it is replaced with the new card.
  void add(List<Attestation> attestations) {
    final currentCards = List.of(_attestations);
    final newAttestationTypes = attestations.map((e) => e.attestationType);
    final cardsToKeep =
        currentCards.whereNot((attestation) => newAttestationTypes.contains(attestation.attestationType));
    final newCardList = List.of(cardsToKeep)..addAll(attestations);
    _attestationsSubject.add(newCardList);
  }
}
