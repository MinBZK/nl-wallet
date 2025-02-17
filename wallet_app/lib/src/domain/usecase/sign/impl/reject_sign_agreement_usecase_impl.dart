import '../../../../data/repository/sign/sign_repository.dart';
import '../../../model/result/result.dart';
import '../reject_sign_agreement_usecase.dart';

class RejectSignAgreementUseCaseImpl extends RejectSignAgreementUseCase {
  final SignRepository _signRepository;

  RejectSignAgreementUseCaseImpl(this._signRepository);

  @override
  Future<Result<void>> invoke() async {
    return tryCatch(
      () async => _signRepository.rejectAgreement(),
      'Failed to explicitly reject the agreement',
    );
  }
}
