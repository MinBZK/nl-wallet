import '../../../../data/repository/issuance/issuance_repository.dart';
import '../accept_issuance_usecase.dart';

class AcceptIssuanceUseCaseImpl extends AcceptIssuanceUseCase {
  final IssuanceRepository _issuanceRepository;

  AcceptIssuanceUseCaseImpl(this._issuanceRepository);

  @override
  Future<void> invoke(Iterable<String> cardDocTypes) async {
    await _issuanceRepository.acceptIssuance(cardDocTypes);
  }
}
