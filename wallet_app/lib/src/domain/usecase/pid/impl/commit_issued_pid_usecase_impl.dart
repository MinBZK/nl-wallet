import '../../../../data/repository/pid/pid_repository.dart';
import '../../../model/result/result.dart';
import '../commit_issued_pid_usecase.dart';

class CommitIssuedPidUseCaseImpl extends CommitIssuedPidUseCase {
  final PidRepository _pidRepository;

  CommitIssuedPidUseCaseImpl(this._pidRepository);

  @override
  Future<Result<TransferState>> invoke(String pin) => tryCatch(
    () => _pidRepository.commitIssuedPid(pin),
    'Failed to commit issued pid',
  );
}
