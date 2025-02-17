import 'package:fimber/fimber.dart';

import '../../../../data/repository/sign/sign_repository.dart';
import '../../../../util/extension/core_error_extension.dart';
import '../../../../util/extension/wallet_instruction_result_extension.dart';
import '../../../../wallet_core/error/core_error.dart';
import '../../../model/result/application_error.dart';
import '../../../model/result/result.dart';
import '../accept_sign_agreement_usecase.dart';

class AcceptSignAgreementUseCaseImpl extends AcceptSignAgreementUseCase {
  final SignRepository _signRepository;

  AcceptSignAgreementUseCaseImpl(this._signRepository);

  @override
  Future<Result<String?>> invoke(String pin) async {
    try {
      final result = await _signRepository.acceptAgreement(pin);
      return result.asApplicationResult();
    } on CoreError catch (ex) {
      Fimber.e('Could not sign agreement', ex: ex);
      return Result.error(await ex.asApplicationError());
    } catch (ex) {
      Fimber.e('Could not sign agreement', ex: ex);
      return Result.error(GenericError(ex.toString(), sourceError: ex));
    }
  }
}
