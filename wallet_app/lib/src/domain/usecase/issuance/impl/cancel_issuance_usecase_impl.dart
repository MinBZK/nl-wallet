import '../../../../data/repository/issuance/issuance_repository.dart';
import '../cancel_issuance_usecase.dart';

class CancelIssuanceUseCaseImpl extends CancelIssuanceUseCase {
  final IssuanceRepository _issuanceRepository;

  CancelIssuanceUseCaseImpl(this._issuanceRepository);

  @override
  Future<void> invoke() => _issuanceRepository.cancelIssuance();
}
