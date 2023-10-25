import '../../../../bridge_generated.dart';
import '../../../domain/model/attribute/data_attribute.dart';

export '../../../domain/model/pid/pid_issuance_status.dart';

abstract class PidRepository {
  Future<String> getPidIssuanceUrl();

  /// Continue the pidIssuance process, the stream exposes a (localised) list of preview attributes
  Stream<List<DataAttribute>> continuePidIssuance(Uri uri);

  Future<void> cancelPidIssuance();

  Future<WalletInstructionResult> acceptOfferedPid(String pin);

  Future<void> rejectOfferedPid();
}
