import '../../../domain/model/attribute/data_attribute.dart';

abstract class DataAttributeRepository {
  Future<List<DataAttribute>?> getAll(String cardId);

  Future<DataAttribute?> find(AttributeType type);
}
