import '../../../data/repository/card/wallet_card_repository.dart';
import '../../model/attribute/attribute.dart';
import '../../model/attribute/data_attribute.dart';
import '../../model/attribute/requested_attribute.dart';
import '../../model/wallet_card.dart';

class GetRequestedAttributesFromWalletUseCase {
  final WalletCardRepository repository;

  GetRequestedAttributesFromWalletUseCase(this.repository);

  Future<List<Attribute>> invoke(List<RequestedAttribute> requestedAttributes) async {
    final List<WalletCard> cards = await repository.readAll();

    List<RequestedAttribute> remaining = List.of(requestedAttributes);
    List<DataAttribute> found = [];
    List<Attribute> results = [];
    do {
      found = _findAttributes(cards, remaining);
      remaining = _getRemainingAttributes(found, remaining);
      results.addAll(found);
    } while (found.isNotEmpty);

    results.addAll(remaining); // Add remaining attributes

    return _sortResultAttributes(results, requestedAttributes);
  }

  /// Finds [DataAttribute]s on a single card, containing all (or most) [RequestedAttribute]s
  List<DataAttribute> _findAttributes(List<WalletCard> cards, List<RequestedAttribute> requestedAttributes) {
    final Set<AttributeType> findTypes = requestedAttributes.map((e) => e.type).toSet();

    List<DataAttribute> results = [];
    for (WalletCard card in cards) {
      final Set<AttributeType> cardAttributeTypes = card.attributes.map((e) => e.type).toSet();
      final Set<AttributeType> intersection = findTypes.intersection(cardAttributeTypes);

      if (intersection.length > results.length) {
        results = card.attributes.where((element) => intersection.contains(element.type)).toList();
      }
    }
    return results;
  }

  /// Removes found attributes from remaining list
  List<RequestedAttribute> _getRemainingAttributes(List<Attribute> found, List<RequestedAttribute> attributes) {
    final List<AttributeType> foundTypes = found.map((e) => e.type).toList();
    final List<RequestedAttribute> remaining = List.of(attributes);
    remaining.removeWhere((element) => foundTypes.contains(element.type));
    return remaining;
  }

  /// Sorts result [List<Attribute>] list based on [List<RequestedAttribute>] order
  List<Attribute> _sortResultAttributes(List<Attribute> results, List<RequestedAttribute> requestedAttributes) {
    return [
      for (RequestedAttribute requestedAttribute in requestedAttributes)
        results.singleWhere((element) => element.type == requestedAttribute.type)
    ];
  }
}
