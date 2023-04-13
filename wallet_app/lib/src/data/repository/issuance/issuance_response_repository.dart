import '../../../domain/model/issuance_response.dart';

abstract class IssuanceResponseRepository {
  Future<IssuanceResponse> read(String issuanceRequestId);
}
