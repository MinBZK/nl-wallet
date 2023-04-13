import '../../../../data/repository/card/data_attribute_repository.dart';
import '../../../../wallet_constants.dart';
import '../../../model/attribute/data_attribute.dart';
import '../get_wallet_card_data_attributes_usecase.dart';

class GetWalletCardDataAttributesUseCaseImpl implements GetWalletCardDataAttributesUseCase {
  final DataAttributeRepository dataAttributeRepository;

  GetWalletCardDataAttributesUseCaseImpl(this.dataAttributeRepository);

  @override
  Future<List<DataAttribute>> invoke(String cardId) async {
    await Future.delayed(kDefaultMockDelay);
    final attributes = await dataAttributeRepository.getAll(cardId);
    if (attributes == null) throw Exception('No attributes found for card with id: $cardId');
    return attributes;
  }
}
