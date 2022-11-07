import '../../../domain/model/data_attribute.dart';

abstract class WalletCardDataAttributeRepository {
  Future<List<DataAttribute>> getAll(String cardId);
}
