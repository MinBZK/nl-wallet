import '../../../../data/repository/issuance/issuance_repository.dart';
import '../../../model/issuance/start_issuance_result.dart';
import '../../../model/result/result.dart';
import '../start_issuance_usecase.dart';

class StartIssuanceUseCaseImpl extends StartIssuanceUseCase {
  final IssuanceRepository _issuanceRepository;

  StartIssuanceUseCaseImpl(this._issuanceRepository);

  @override
  Future<Result<StartIssuanceResult>> invoke(String issuanceUri) async {
    return tryCatch(
      () async => _issuanceRepository.startIssuance(issuanceUri),
      'Failed to start issuance',
    );
  }
}
