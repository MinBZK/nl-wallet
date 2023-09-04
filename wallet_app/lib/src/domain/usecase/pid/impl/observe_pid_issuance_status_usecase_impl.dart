import 'dart:async';

import '../../../../data/repository/pid/pid_repository.dart';
import '../observe_pid_issuance_status_usecase.dart';

class ObservePidIssuanceStatusUseCaseImpl implements ObservePidIssuanceStatusUseCase {
  final PidRepository _pidRepository;

  ObservePidIssuanceStatusUseCaseImpl(this._pidRepository);

  @override
  Stream<PidIssuanceStatus> invoke() {
    return _pidRepository.observePidIssuanceStatus().where((status) => status != PidIssuanceStatus.idle);
  }
}
