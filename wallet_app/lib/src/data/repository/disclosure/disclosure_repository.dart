import '../../../domain/model/disclosure/start_disclosure_result.dart';

export '../../../domain/model/disclosure/start_disclosure_result.dart';

abstract class DisclosureRepository {
  Future<StartDisclosureResult> startDisclosure(String disclosureUri, {required bool isQrCode});

  Future<StartDisclosureResult> continueCloseProximityDisclosure();

  Future<String?> acceptDisclosure(String pin, List<int> selectedIndices);
}
