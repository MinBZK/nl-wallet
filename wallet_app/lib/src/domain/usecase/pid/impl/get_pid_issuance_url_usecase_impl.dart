import '../../../../data/repository/pid/pid_repository.dart';
import '../../../model/result/result.dart';
import '../get_pid_issuance_url_usecase.dart';

class GetPidIssuanceUrlUseCaseImpl extends GetPidIssuanceUrlUseCase {
  final PidRepository _pidRepository;

  GetPidIssuanceUrlUseCaseImpl(this._pidRepository);

  @override
  Future<Result<String>> invoke() async {
    return tryCatch(
      () async => _pidRepository.getPidIssuanceUrl(),
      'Failed to get pid issuance url',
    );
  }
}
