import '../../model/attribute/attribute.dart';
import '../../model/attribute/missing_attribute.dart';

abstract class GetRequestedAttributesFromWalletUseCase {
  Future<List<Attribute>> invoke(List<MissingAttribute> requestedAttributes);
}
