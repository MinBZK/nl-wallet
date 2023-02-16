import '../../model/issuance_response.dart';

abstract class GetPidIssuanceResponseUseCase {
  Future<IssuanceResponse> invoke();
}
