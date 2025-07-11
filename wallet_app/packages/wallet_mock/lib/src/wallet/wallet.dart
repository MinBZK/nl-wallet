import 'package:collection/collection.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet_core/core.dart';

class Wallet {
  final BehaviorSubject<bool> _isLockedSubject = BehaviorSubject.seeded(true);
  final BehaviorSubject<List<AttestationPresentation>> _attestationsSubject = BehaviorSubject.seeded([]);

  Wallet();

  Stream<bool> get lockedStream => _isLockedSubject.stream;

  Stream<List<AttestationPresentation>> get attestationsStream => _attestationsSubject.stream;

  List<AttestationPresentation> get _attestations => _attestationsSubject.value;

  List<AttestationAttribute> get _allAttributes =>
      _attestations.map((attestation) => attestation.attributes).flattened.toList();

  bool get isEmpty => _attestations.isEmpty;

  bool containsAttributes(Iterable<String> keys) => keys.every(containsAttribute);

  bool containsAttribute(String attributeKey) {
    return _attestations.any((element) => element.attributes.any((element) => element.key == attributeKey));
  }

  AttestationAttribute? findAttribute(String key) =>
      _allAttributes.firstWhereOrNull((attribute) => attribute.key == key);

  List<AttestationPresentation> getRequestedAttestations(Iterable<String> keys) {
    final allRequestedAttributes = keys.map(findAttribute).nonNulls;
    final cardToAttributes = allRequestedAttributes
        .groupListsBy((attribute) => _attestations.firstWhere((card) => card.attributes.contains(attribute)));
    return cardToAttributes.entries
        .map(
          (e) => AttestationPresentation(
            identity: e.key.identity,
            attestationType: e.key.attestationType,
            displayMetadata: e.key.displayMetadata,
            issuer: e.key.issuer,
            attributes: e.value,
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

  /// Adds the cards to the wallet, if a card with the same attestationType already exists, it is replaced with the new card.
  void add(List<AttestationPresentation> attestations) {
    final currentCards = List.of(_attestations);
    final newAttestationTypes = attestations.map((e) => e.attestationType);
    final cardsToKeep =
        currentCards.whereNot((attestation) => newAttestationTypes.contains(attestation.attestationType));
    final newCardList = List.of(cardsToKeep)..addAll(attestations);
    _attestationsSubject.add(newCardList);
  }

  /// Checks whether the attestion already exists in the user's wallet. Currently solely based on the (legacy / mock) attestationType attribute.
  bool containsAttestation(AttestationPresentation attestation) {
    return _attestations.any((it) => it.attestationType == attestation.attestationType);
  }
}
