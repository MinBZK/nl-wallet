import '../../../../data/repository/issuance/issuance_response_repository.dart';
import '../../../model/issuance_response.dart';
import '../get_pid_issuance_response_usecase.dart';

const _kPidCardId = 'PID_1';

class GetPidIssuanceResponseUseCaseImpl implements GetPidIssuanceResponseUseCase {
  final IssuanceResponseRepository issuanceResponseRepository;

  const GetPidIssuanceResponseUseCaseImpl(this.issuanceResponseRepository);

  @override
  Future<IssuanceResponse> invoke() async => await issuanceResponseRepository.read(_kPidCardId);
}
