import '../../../../data/repository/pid/pid_repository.dart';
import '../get_pid_issuance_url_usecase.dart';

class GetPidIssuanceUrlUseCaseImpl implements GetPidIssuanceUrlUseCase {
  final PidRepository _pidRepository;

  GetPidIssuanceUrlUseCaseImpl(this._pidRepository);

  @override
  Future<String> invoke() => _pidRepository.getPidIssuanceUrl();
}
