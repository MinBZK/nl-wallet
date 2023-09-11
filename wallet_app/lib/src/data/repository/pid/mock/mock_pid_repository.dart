import 'package:fimber/fimber.dart';

import '../../../../../bridge_generated.dart';
import '../pid_repository.dart';

class MockPidRepository extends PidRepository {
  @override
  Future<String> getPidIssuanceUrl() async => 'mock://auth_url';

  @override
  Future<void> cancelPidIssuance() async => Fimber.d('Canceled PidIssuance');

  @override
  void notifyPidIssuanceStateUpdate(PidIssuanceEvent? event) {
    Fimber.d('Received PidIssuance update: $event');
  }

  @override
  Stream<PidIssuanceStatus> observePidIssuanceStatus() => const Stream.empty();
}
