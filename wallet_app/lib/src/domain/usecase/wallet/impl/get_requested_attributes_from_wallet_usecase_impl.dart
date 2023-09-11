import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../model/attribute/attribute.dart';
import '../../../model/attribute/data_attribute.dart';
import '../../../model/attribute/requested_attribute.dart';
import '../../../model/wallet_card.dart';
import '../get_requested_attributes_from_wallet_usecase.dart';

class GetRequestedAttributesFromWalletUseCaseImpl implements GetRequestedAttributesFromWalletUseCase {
  final WalletCardRepository repository;

  GetRequestedAttributesFromWalletUseCaseImpl(this.repository);

  @override
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
    final Set<AttributeKey> findTypes = requestedAttributes.map((e) => e.key).toSet();

    List<DataAttribute> results = [];
    for (WalletCard card in cards) {
      final Set<AttributeKey> cardAttributeKeys = card.attributes.map((e) => e.key).toSet();
      final Set<AttributeKey> intersection = findTypes.intersection(cardAttributeKeys);

      if (intersection.length > results.length) {
        results = card.attributes.where((element) => intersection.contains(element.key)).toList();
      }
    }
    return results;
  }

  /// Removes found attributes from remaining list
  List<RequestedAttribute> _getRemainingAttributes(List<Attribute> found, List<RequestedAttribute> attributes) {
    final List<AttributeKey> foundKeys = found.map((e) => e.key).toList();
    final List<RequestedAttribute> remaining = List.of(attributes);
    remaining.removeWhere((element) => foundKeys.contains(element.key));
    return remaining;
  }

  /// Sorts result [List<Attribute>] list based on [List<RequestedAttribute>] order
  List<Attribute> _sortResultAttributes(List<Attribute> results, List<RequestedAttribute> requestedAttributes) {
    return [
      for (RequestedAttribute requestedAttribute in requestedAttributes)
        results.singleWhere((element) => element.key == requestedAttribute.key)
    ];
  }
}
