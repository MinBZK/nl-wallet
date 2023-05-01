import '../../model/attribute/data_attribute.dart';
import '../../model/attribute/requested_attribute.dart';
import '../../model/wallet_card.dart';

abstract class GetRequestedAttributesWithCardUseCase {
  /// Looks for the [requestedAttributes] in the user's wallet. Note that the list of returned [DataAttributes]
  /// can be smaller, in case not all [requestedAttributes] are found.
  Future<Map<WalletCard, List<DataAttribute>>> invoke(List<RequestedAttribute> requestedAttributes);
}
