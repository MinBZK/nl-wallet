import '../../../../data/repository/card/data_attribute_repository.dart';
import '../../../model/attribute/attribute.dart';
import '../get_first_names_usecase.dart';

const kNameKeyCandidates = ['given_name', 'pid.firstNames', 'mock.firstNames'];

class GetFirstNamesUseCaseImpl implements GetFirstNamesUseCase {
  final DataAttributeRepository _dataAttributeRepository;

  GetFirstNamesUseCaseImpl(this._dataAttributeRepository);

  @override
  Future<String> invoke() async {
    for (final candidate in kNameKeyCandidates) {
      final attribute = await _dataAttributeRepository.find(candidate);
      if (attribute != null && attribute.value is StringValue) return (attribute.value as StringValue).value;
    }
    throw UnsupportedError('First name not found in available attributes');
  }
}
