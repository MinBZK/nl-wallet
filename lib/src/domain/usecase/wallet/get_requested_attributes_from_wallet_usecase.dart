import '../../../data/repository/card/data_attribute_repository.dart';
import '../../model/attribute/attribute.dart';
import '../../model/attribute/requested_attribute.dart';

class GetRequestedAttributesFromWalletUseCase {
  final DataAttributeRepository repository;

  GetRequestedAttributesFromWalletUseCase(this.repository);

  Future<List<Attribute>> invoke(List<RequestedAttribute> requestedAttributes) async {
    final resolvedAttributes = requestedAttributes.map((attribute) async {
      return await repository.find(attribute.type) ?? attribute;
    });
    return Future.wait(resolvedAttributes);
  }
}
