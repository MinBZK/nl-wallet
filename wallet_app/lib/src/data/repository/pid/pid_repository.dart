import '../../../../bridge_generated.dart';
import '../../../domain/model/pid/pid_issuance_status.dart';

export '../../../domain/model/pid/pid_issuance_status.dart';

abstract class PidRepository {
  Future<String> getPidIssuanceUrl();

  void notifyPidIssuanceStateUpdate(PidIssuanceEvent? event);

  Stream<PidIssuanceStatus> observePidIssuanceStatus();
}
