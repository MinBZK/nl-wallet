import '../../model/issuance/continue_issuance_result.dart';

abstract class ContinueIssuanceUseCase {
  Future<ContinueIssuanceResult> invoke();
}
