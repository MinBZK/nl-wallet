import '../../../domain/model/data_attribute.dart';

abstract class DataAttributeRepository {
  Future<List<DataAttribute>?> getAll(String cardId);
}
