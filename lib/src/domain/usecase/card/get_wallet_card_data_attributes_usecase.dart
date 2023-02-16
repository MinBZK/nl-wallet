import '../../model/attribute/data_attribute.dart';

abstract class GetWalletCardDataAttributesUseCase {
  Future<List<DataAttribute>> invoke(String cardId);
}
