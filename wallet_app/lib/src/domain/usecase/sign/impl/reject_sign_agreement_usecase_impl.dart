import '../../../../data/repository/sign/sign_repository.dart';
import '../reject_sign_agreement_usecase.dart';

class RejectSignAgreementUseCaseImpl implements RejectSignAgreementUseCase {
  final SignRepository _signRepository;

  RejectSignAgreementUseCaseImpl(this._signRepository);

  @override
  Future<void> invoke() => _signRepository.rejectAgreement();
}
