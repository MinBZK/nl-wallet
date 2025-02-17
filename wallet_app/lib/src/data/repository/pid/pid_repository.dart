import 'package:wallet_core/core.dart';

import '../../../domain/model/attribute/attribute.dart';

abstract class PidRepository {
  Future<String> getPidIssuanceUrl();

  /// Continue the pidIssuance, returns a preview of all the attributes that will be added if the pid is accepted.
  Future<List<DataAttribute>> continuePidIssuance(String uri);

  Future<void> cancelPidIssuance();

  Future<bool> hasActivePidIssuanceSession();

  Future<WalletInstructionResult> acceptOfferedPid(String pin);
}
