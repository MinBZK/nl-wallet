import '../../model/attribute/attribute.dart';
import '../../model/attribute/requested_attribute.dart';

abstract class GetRequestedAttributesFromWalletUseCase {
  Future<List<Attribute>> invoke(List<RequestedAttribute> requestedAttributes);
}
