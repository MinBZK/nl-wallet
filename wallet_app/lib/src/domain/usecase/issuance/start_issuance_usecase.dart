import '../../model/issuance/start_issuance_result.dart';

abstract class StartIssuanceUseCase {
  Future<StartIssuanceResult> invoke(String issuanceUri);
}
