import 'package:wallet_core/core.dart' hide StartDisclosureResult;

import '../../../domain/model/disclosure/start_disclosure_result.dart';

export '../../../domain/model/disclosure/start_disclosure_result.dart';
export '../../../feature/disclosure/model/disclosure_request.dart';

abstract class DisclosureRepository {
  Future<StartDisclosureResult> startDisclosure(String disclosureUri);

  Future<void> cancelDisclosure();

  Future<WalletInstructionResult> acceptDisclosure(String pin);
}
