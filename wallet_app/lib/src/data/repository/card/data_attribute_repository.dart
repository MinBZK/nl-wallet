import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/attribute/data_attribute.dart';

abstract class DataAttributeRepository {
  Future<List<DataAttribute>?> getAll(String cardId);

  Future<DataAttribute?> find(AttributeKey type);
}
