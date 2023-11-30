import 'package:wallet_core/core.dart';

import '../../../domain/model/start_sign_result/start_sign_result.dart';

abstract class SignRepository {
  Future<StartSignResult> startSigning(String signUri);

  Future<WalletInstructionResult> acceptAgreement(String pin);

  Future<void> rejectAgreement();
}
