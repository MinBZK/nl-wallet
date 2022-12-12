import '../../../data/repository/issuance/issuance_response_repository.dart';
import '../../model/issuance_response.dart';

const _kPidCardId = 'PID_1';

class GetPidIssuanceResponseUseCase {
  final IssuanceResponseRepository issuanceResponseRepository;

  const GetPidIssuanceResponseUseCase(this.issuanceResponseRepository);

  Future<IssuanceResponse> invoke() async => await issuanceResponseRepository.read(_kPidCardId);
}
