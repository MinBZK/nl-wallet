import '../../../../../bridge_generated.dart';
import '../../../../data/repository/pid/pid_repository.dart';
import '../update_pid_issuance_status_usecase.dart';

class UpdatePidIssuanceStatusUseCaseImpl extends UpdatePidIssuanceStatusUseCase {
  final PidRepository _pidRepository;

  UpdatePidIssuanceStatusUseCaseImpl(this._pidRepository);

  @override
  Future<void> invoke(PidIssuanceEvent state) async {
    _pidRepository.notifyPidIssuanceStateUpdate(state);
  }
}
