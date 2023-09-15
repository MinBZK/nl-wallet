import '../../../../data/repository/pid/pid_repository.dart';
import '../reject_offered_pid_usecase.dart';

class RejectOfferedPidUseCaseImpl implements RejectOfferedPidUseCase {
  final PidRepository _pidRepository;

  RejectOfferedPidUseCaseImpl(this._pidRepository);

  @override
  Future<void> invoke() => _pidRepository.rejectOfferedPid();
}
