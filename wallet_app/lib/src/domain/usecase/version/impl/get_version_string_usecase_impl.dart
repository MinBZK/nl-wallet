import '../../../../data/repository/version/version_string_repository.dart';
import '../get_version_string_usecase.dart';

class GetVersionStringUseCaseImpl implements GetVersionStringUseCase {
  final VersionStringRepository _versionStringRepository;

  GetVersionStringUseCaseImpl(this._versionStringRepository);

  @override
  Future<String> invoke() => _versionStringRepository.versionString;
}
