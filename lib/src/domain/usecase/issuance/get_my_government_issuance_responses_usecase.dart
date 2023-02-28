import '../../model/issuance_response.dart';

abstract class GetMyGovernmentIssuanceResponsesUseCase {
  Future<List<IssuanceResponse>> invoke();
}
