import '../../model/issuance_response.dart';

abstract class GetIssuanceResponseUseCase {
  Future<IssuanceResponse> invoke(String issuanceRequestId);
}
