import '../../../../data/repository/issuance/issuance_repository.dart';
import '../../../model/issuance/start_issuance_result.dart';
import '../start_issuance_usecase.dart';

class StartIssuanceUseCaseImpl extends StartIssuanceUseCase {
  final IssuanceRepository _issuanceRepository;

  StartIssuanceUseCaseImpl(this._issuanceRepository);

  @override
  Future<StartIssuanceResult> invoke(String issuanceUri) {
    return _issuanceRepository.startIssuance(issuanceUri);
  }
}
