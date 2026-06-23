import '../../../../data/repository/issuance/issuance_repository.dart';
import '../../../model/card/wallet_card.dart';
import '../../../model/result/result.dart';
import '../continue_issuance_usecase.dart';

class ContinueIssuanceUseCaseImpl extends ContinueIssuanceUseCase {
  final IssuanceRepository _issuanceRepository;

  ContinueIssuanceUseCaseImpl(this._issuanceRepository);

  @override
  Future<Result<List<WalletCard>>> invoke(String uri) async {
    return tryCatch(
      () async => _issuanceRepository.continueIssuance(uri),
      'Failed to continue issuance',
    );
  }
}
