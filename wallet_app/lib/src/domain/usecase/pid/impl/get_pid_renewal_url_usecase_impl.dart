import '../../../../data/repository/pid/pid_repository.dart';
import '../../../model/result/result.dart';
import '../get_pid_renewal_url_usecase.dart';

class GetPidRenewalUrlUseCaseImpl extends GetPidRenewalUrlUseCase {
  final PidRepository _pidRepository;

  GetPidRenewalUrlUseCaseImpl(this._pidRepository);

  @override
  Future<Result<String>> invoke() async {
    return tryCatch(
      () async => _pidRepository.getPidRenewalUrl(),
      'Failed to get pid renewal url',
    );
  }
}
