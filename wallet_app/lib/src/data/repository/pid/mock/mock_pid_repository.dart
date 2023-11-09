import 'package:fimber/fimber.dart';

import '../../../../../bridge_generated.dart';
import '../../../../domain/model/attribute/data_attribute.dart';
import '../pid_repository.dart';

class MockPidRepository extends PidRepository {
  @override
  Future<String> getPidIssuanceUrl() async => 'mock://auth_url';

  @override
  Future<void> cancelPidIssuance() async => Fimber.d('Canceled PidIssuance');

  @override
  Future<WalletInstructionResult> acceptOfferedPid(String pin) => throw UnimplementedError();

  @override
  Future<void> rejectOfferedPid() async => Fimber.d('Pid declined');

  @override
  Future<List<DataAttribute>> continuePidIssuance(String uri) {
    throw UnimplementedError('This method should not be called on mock builds');
  }
}
