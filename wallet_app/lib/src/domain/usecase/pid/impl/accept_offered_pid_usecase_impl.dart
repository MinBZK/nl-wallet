import '../../../../data/repository/pid/pid_repository.dart';
import '../../../model/result/result.dart';
import '../accept_offered_pid_usecase.dart';

class AcceptOfferedPidUseCaseImpl extends AcceptOfferedPidUseCase {
  final PidRepository _pidRepository;

  AcceptOfferedPidUseCaseImpl(this._pidRepository);

  @override
  Future<Result<TransferState>> invoke(String pin) async {
    return tryCatch(
      () async => _pidRepository.acceptIssuance(pin),
      'Failed to accept pid',
    );
  }
}
