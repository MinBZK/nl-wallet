import 'package:wallet_core/core.dart' hide StartDisclosureResult;

import '../../../domain/model/issuance/continue_issuance_result.dart';
import '../../../domain/model/issuance/start_issuance_result.dart';

export '../../../domain/model/disclosure/start_disclosure_result.dart';

abstract class IssuanceRepository {
  Future<StartIssuanceResult> startIssuance(String disclosureUri);

  Future<WalletInstructionResult> discloseForIssuance(String pin);

  /// Will only have data if [discloseForIssuance] returned OK
  Future<ContinueIssuanceResult> proceedIssuance();

  Future<void> acceptIssuance(Iterable<String> cardDocTypes);

  Future<void> cancelIssuance();
}
