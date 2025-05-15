import '../../../../data/repository/pid/pid_repository.dart';
import '../../../model/result/result.dart';
import '../cancel_pid_issuance_usecase.dart';

class CancelPidIssuanceUseCaseImpl extends CancelPidIssuanceUseCase {
  final PidRepository _pidRepository;

  CancelPidIssuanceUseCaseImpl(this._pidRepository);

  @override
  Future<Result<bool>> invoke() async {
    return tryCatch(
      () async {
        final hasActiveSession = await _pidRepository.hasActiveIssuanceSession();
        if (hasActiveSession) await _pidRepository.cancelIssuance();
        return hasActiveSession;
      },
      'Failed to cancel pid issuance session',
    );
  }
}
