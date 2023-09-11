import '../../../../data/repository/pid/pid_repository.dart';
import '../cancel_pid_issuance_usecase.dart';

class CancelPidIssuanceUseCaseImpl implements CancelPidIssuanceUseCase {
  final PidRepository _pidRepository;

  CancelPidIssuanceUseCaseImpl(this._pidRepository);

  @override
  Future<void> invoke() => _pidRepository.cancelPidIssuance();
}
