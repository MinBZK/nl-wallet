import '../../../../../environment.dart';
import '../../../../data/repository/card/data_attribute_repository.dart';
import '../get_first_names_usecase.dart';

class GetFirstNamesUseCaseImpl implements GetFirstNamesUseCase {
  final DataAttributeRepository repository;

  GetFirstNamesUseCaseImpl(this.repository);

  @override
  Future<String> invoke() async {
    if (!Environment.mockRepositories) throw UnimplementedError('TODO: For now only supported on mock builds');
    final attribute = await repository.find('mock.firstNames');
    return attribute!.value;
  }
}
