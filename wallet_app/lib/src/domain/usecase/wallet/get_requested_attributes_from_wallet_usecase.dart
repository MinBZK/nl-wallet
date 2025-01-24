import '../../model/attribute/attribute.dart';

abstract class GetRequestedAttributesFromWalletUseCase {
  Future<List<Attribute>> invoke(List<MissingAttribute> requestedAttributes);
}
