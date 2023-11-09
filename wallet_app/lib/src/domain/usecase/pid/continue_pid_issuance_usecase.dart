import '../../../data/repository/pid/pid_repository.dart';

abstract class ContinuePidIssuanceUseCase {
  Future<PidIssuanceStatus> invoke(String uri);
}
