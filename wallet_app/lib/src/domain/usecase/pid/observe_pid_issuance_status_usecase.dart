import '../../../data/repository/pid/pid_repository.dart';

abstract class ObservePidIssuanceStatusUseCase {
  Stream<PidIssuanceStatus> invoke();
}
