import 'package:wallet_core/core.dart' hide StartDisclosureResult;

import '../../../domain/model/disclosure/start_disclosure_result.dart';

export '../../../domain/model/disclosure/start_disclosure_result.dart';

abstract class DisclosureRepository {
  Future<StartDisclosureResult> startDisclosure(String disclosureUri, {required bool isQrCode});

  Future<String?> cancelDisclosure();

  Future<bool> hasActiveDisclosureSession();

  Future<AcceptDisclosureResult> acceptDisclosure(String pin);
}
