import '../../../../data/repository/issuance/issuance_repository.dart';
import '../../../../feature/issuance/argument/issuance_screen_argument.dart';
import '../../../model/issuance/start_issuance_result.dart';
import '../../../model/result/result.dart';
import '../start_issuance_usecase.dart';

class StartIssuanceUseCaseImpl extends StartIssuanceUseCase {
  final IssuanceRepository _issuanceRepository;

  StartIssuanceUseCaseImpl(this._issuanceRepository);

  @override
  Future<Result<StartIssuanceResult>> invoke(
    String issuanceUri, {
    bool isQrCode = false,
    required IssuanceType type,
  }) async {
    return tryCatch(
      () async {
        switch (type) {
          case IssuanceType.disclosureBasedIssuance:
            return _issuanceRepository.startIssuance(issuanceUri, isQrCode: isQrCode);
          case IssuanceType.credentialOffer:
            return _issuanceRepository.startIssuanceFromOffer(issuanceUri, isQrCode: isQrCode);
          case IssuanceType.authorizationCallback:
            throw UnsupportedError('$type should rely on continueIssuance()');
        }
      },
      'Failed to start issuance',
    );
  }
}
