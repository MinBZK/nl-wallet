import '../../../data/repository/card/data_attribute_repository.dart';
import '../../../wallet_constants.dart';
import '../../model/attribute/data_attribute.dart';

class GetWalletCardDataAttributesUseCase {
  final DataAttributeRepository dataAttributeRepository;

  GetWalletCardDataAttributesUseCase(this.dataAttributeRepository);

  Future<List<DataAttribute>> invoke(String cardId) async {
    await Future.delayed(kDefaultMockDelay);
    final attributes = await dataAttributeRepository.getAll(cardId);
    if (attributes == null) throw Exception('No attributes found for card with id: $cardId');
    return attributes;
  }
}
