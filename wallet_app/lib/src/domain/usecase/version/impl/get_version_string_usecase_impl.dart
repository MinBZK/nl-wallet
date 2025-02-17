import '../../../../data/repository/version/version_string_repository.dart';
import '../../../model/result/result.dart';
import '../get_version_string_usecase.dart';

class GetVersionStringUseCaseImpl extends GetVersionStringUseCase {
  final VersionStringRepository _versionStringRepository;

  GetVersionStringUseCaseImpl(this._versionStringRepository);

  @override
  Future<Result<String>> invoke() async {
    return tryCatch(
      () async => _versionStringRepository.versionString,
      'Failed to get version',
    );
  }
}
