import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../model/attribute/attribute.dart';
import '../../../model/wallet_card.dart';
import '../get_requested_attributes_from_wallet_usecase.dart';

class GetRequestedAttributesFromWalletUseCaseImpl implements GetRequestedAttributesFromWalletUseCase {
  final WalletCardRepository repository;

  GetRequestedAttributesFromWalletUseCaseImpl(this.repository);

  @override
  Future<List<Attribute>> invoke(List<MissingAttribute> requestedAttributes) async {
    final List<WalletCard> cards = await repository.readAll();

    List<MissingAttribute> remaining = List.of(requestedAttributes);
    List<DataAttribute> found = [];
    final List<Attribute> results = [];
    do {
      found = _findAttributes(cards, remaining);
      remaining = _getRemainingAttributes(found, remaining);
      results.addAll(found);
    } while (found.isNotEmpty);

    results.addAll(remaining); // Add remaining attributes

    return _sortResultAttributes(results, requestedAttributes);
  }

  /// Finds [DataAttribute]s on a single card, containing all (or most) [MissingAttribute]s
  List<DataAttribute> _findAttributes(List<WalletCard> cards, List<MissingAttribute> requestedAttributes) {
    final Set<AttributeKey> findTypes = requestedAttributes.map((e) => e.key).toSet();

    List<DataAttribute> results = [];
    for (final WalletCard card in cards) {
      final Set<AttributeKey> cardAttributeKeys = card.attributes.map((e) => e.key).toSet();
      final Set<AttributeKey> intersection = findTypes.intersection(cardAttributeKeys);

      if (intersection.length > results.length) {
        results = card.attributes.where((element) => intersection.contains(element.key)).toList();
      }
    }
    return results;
  }

  /// Removes found attributes from remaining list
  List<MissingAttribute> _getRemainingAttributes(List<Attribute> found, List<MissingAttribute> attributes) {
    final List<AttributeKey> foundKeys = found.map((e) => e.key).toList();
    final List<MissingAttribute> remaining = List.of(attributes);
    remaining.removeWhere((element) => foundKeys.contains(element.key));
    return remaining;
  }

  /// Sorts result [List<Attribute>] list based on [List<RequestedAttribute>] order
  List<Attribute> _sortResultAttributes(List<Attribute> results, List<MissingAttribute> requestedAttributes) {
    return [
      for (final MissingAttribute requestedAttribute in requestedAttributes)
        results.singleWhere((element) => element.key == requestedAttribute.key),
    ];
  }
}
