import '../../../data/repository/card/data_attribute_repository.dart';
import '../../model/data_attribute.dart';
import '../../model/requested_attribute.dart';

class GetRequestedAttributesFromWalletUseCase {
  final DataAttributeRepository repository;

  GetRequestedAttributesFromWalletUseCase(this.repository);

  Future<List<DataAttribute>> invoke(List<RequestedAttribute> requestedAttributes) async {
    final resolvedAttributes = requestedAttributes.map((attribute) async {
      return await repository.find(attribute.type) ??
          DataAttribute(valueType: attribute.valueType, label: attribute.name, value: null);
    });
    return Future.wait(resolvedAttributes);
  }
}
