import '../../../../bridge_generated.dart';

abstract class PidRepository {
  Future<String> getPidIssuanceUrl();

  void notifyPidIssuanceStateUpdate(PidIssuanceEvent? event);

  Stream<PidIssuanceStatus> observePidIssuanceStatus();
}

enum PidIssuanceStatus { idle, authenticating, success, error }
