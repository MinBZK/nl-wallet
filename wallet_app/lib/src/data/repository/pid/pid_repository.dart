import 'package:wallet_core/core.dart';

import '../../../domain/model/attribute/attribute.dart';

abstract class PidRepository {
  Future<String> getPidIssuanceUrl();

  Future<String> getPidRenewalUrl();

  /// Continue the pidIssuance, returns a preview of all the attributes that will be added if the pid is accepted.
  Future<List<DataAttribute>> continuePidIssuance(String uri);

  Future<void> cancelIssuance();

  Future<bool> hasActiveIssuanceSession();

  Future<WalletInstructionResult> acceptIssuance(String pin);
}
