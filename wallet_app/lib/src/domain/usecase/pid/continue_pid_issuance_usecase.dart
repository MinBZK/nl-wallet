import '../../../data/repository/pid/pid_repository.dart';

abstract class ContinuePidIssuanceUseCase {
  Stream<PidIssuanceStatus> invoke(Uri uri);
}
