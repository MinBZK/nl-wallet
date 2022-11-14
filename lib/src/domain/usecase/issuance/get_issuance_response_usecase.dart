import '../../../data/repository/issuance/issuance_response_repository.dart';
import '../../../wallet_constants.dart';
import '../../model/issuance_response.dart';

class GetIssuanceResponseUseCase {
  final IssuanceResponseRepository issuanceRepository;

  GetIssuanceResponseUseCase(this.issuanceRepository);

  Future<IssuanceResponse> invoke(String issuanceRequestId) async {
    await Future.delayed(kDefaultMockDelay);
    return issuanceRepository.read(issuanceRequestId);
  }
}
