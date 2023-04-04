import '../../../../data/repository/issuance/issuance_response_repository.dart';
import '../../../../wallet_constants.dart';
import '../../../model/issuance_response.dart';
import '../get_issuance_response_usecase.dart';

class GetIssuanceResponseUseCaseImpl implements GetIssuanceResponseUseCase {
  final IssuanceResponseRepository issuanceRepository;

  GetIssuanceResponseUseCaseImpl(this.issuanceRepository);

  @override
  Future<IssuanceResponse> invoke(String issuanceRequestId) async {
    await Future.delayed(kDefaultMockDelay);
    return issuanceRepository.read(issuanceRequestId);
  }
}
