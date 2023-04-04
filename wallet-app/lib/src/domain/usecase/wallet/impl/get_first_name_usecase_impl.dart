import '../../../../data/repository/card/data_attribute_repository.dart';
import '../../../model/attribute/attribute.dart';
import '../get_first_name_usecase.dart';

class GetFirstNamesUseCaseImpl implements GetFirstNamesUseCase {
  final DataAttributeRepository repository;

  GetFirstNamesUseCaseImpl(this.repository);

  @override
  Future<String> invoke() async {
    final attribute = await repository.find(AttributeType.firstNames);
    return attribute!.value;
  }
}
