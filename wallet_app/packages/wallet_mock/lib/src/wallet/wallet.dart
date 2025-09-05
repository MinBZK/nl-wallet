import 'package:collection/collection.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet_core/core.dart';

import '../util/extension/attestation_presentation_extension.dart';

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

  void lock() => _isLockedSubject.add(true);

  void unlock() => _isLockedSubject.add(false);

  /// Adds the cards to the wallet, if a card with the same attestationType already exists, it is replaced with the new card.
  void add(List<AttestationPresentation> attestations) {
    final retainedAttestations = List.of(_attestations)
      ..retainWhere((oldCard) => attestations.none((newCard) => oldCard.attestationType == newCard.attestationType));

    /// Assign fixed identity to the cards that are now "in" the user's wallet.
    final result = (retainedAttestations + attestations).map((it) => it.fixed());

    _attestationsSubject.add(result.toList());
  }

  /// Checks if the wallet already contains an attestation with the provided type.
  bool containsAttestationType(String attestationType) =>
      _attestations.any((it) => it.attestationType == attestationType);
}
