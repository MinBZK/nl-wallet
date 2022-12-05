import '../../../data/repository/card/data_attribute_repository.dart';
import '../../model/attribute/attribute.dart';

class GetFirstNamesUseCase {
  final DataAttributeRepository repository;

  GetFirstNamesUseCase(this.repository);

  Future<String> invoke() async {
    final attribute = await repository.find(AttributeType.firstNames);
    return attribute!.value;
  }
}
