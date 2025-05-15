import '../../../../data/repository/issuance/issuance_repository.dart';
import '../../../model/result/result.dart';
import '../cancel_issuance_usecase.dart';

class CancelIssuanceUseCaseImpl extends CancelIssuanceUseCase {
  final IssuanceRepository _issuanceRepository;

  CancelIssuanceUseCaseImpl(this._issuanceRepository);

  @override
  Future<Result<String?>> invoke() async {
    return tryCatch(
      () async => _issuanceRepository.cancelIssuance(),
      'Failed to cancel issuance',
    );
  }
}
