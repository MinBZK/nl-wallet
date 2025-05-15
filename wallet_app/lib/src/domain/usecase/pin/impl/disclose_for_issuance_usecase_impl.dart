import '../../../../data/repository/issuance/issuance_repository.dart';
import '../../../model/card/wallet_card.dart';
import '../disclose_for_issuance_usecase.dart';

class DiscloseForIssuanceUseCaseImpl extends DiscloseForIssuanceUseCase {
  final IssuanceRepository _issuanceRepository;

  DiscloseForIssuanceUseCaseImpl(this._issuanceRepository);

  @override
  Future<Result<List<WalletCard>>> invoke(String pin) => tryCatch(
        () => _issuanceRepository.discloseForIssuance(pin),
        'Failed to disclose for issuance',
      );
}
