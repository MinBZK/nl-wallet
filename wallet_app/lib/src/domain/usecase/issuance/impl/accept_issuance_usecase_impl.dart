import '../../../../data/repository/issuance/issuance_repository.dart';
import '../../../model/result/result.dart';
import '../accept_issuance_usecase.dart';

class AcceptIssuanceUseCaseImpl extends AcceptIssuanceUseCase {
  final IssuanceRepository _issuanceRepository;

  AcceptIssuanceUseCaseImpl(this._issuanceRepository);

  @override
  Future<Result<void>> invoke(Iterable<String> cardDocTypes) async {
    return tryCatch(
      () async => _issuanceRepository.acceptIssuance(cardDocTypes),
      'Failed to accept issuance',
    );
  }
}
