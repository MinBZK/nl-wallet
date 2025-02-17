import '../../../../data/repository/issuance/issuance_repository.dart';
import '../../../model/issuance/continue_issuance_result.dart';
import '../../../model/result/result.dart';
import '../continue_issuance_usecase.dart';

class ContinueIssuanceUseCaseImpl extends ContinueIssuanceUseCase {
  final IssuanceRepository _issuanceRepository;

  ContinueIssuanceUseCaseImpl(this._issuanceRepository);

  @override
  Future<Result<ContinueIssuanceResult>> invoke() async {
    return tryCatch(
      () async {
        final proceedResult = await _issuanceRepository.proceedIssuance();
        return ContinueIssuanceResult(proceedResult.cards);
      },
      'Failed to continue issuance',
    );
  }
}
