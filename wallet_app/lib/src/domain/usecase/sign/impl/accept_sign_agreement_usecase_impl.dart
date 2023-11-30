import '../../../../data/repository/sign/sign_repository.dart';
import '../../../../util/extension/wallet_instruction_result_extension.dart';
import '../../../model/pin/check_pin_result.dart';
import '../accept_sign_agreement_usecase.dart';

class AcceptSignAgreementUseCaseImpl implements AcceptSignAgreementUseCase {
  final SignRepository _signRepository;

  AcceptSignAgreementUseCaseImpl(this._signRepository);

  @override
  Future<CheckPinResult> invoke(String pin) async {
    final result = await _signRepository.acceptAgreement(pin);
    return result.asCheckPinResult();
  }
}
