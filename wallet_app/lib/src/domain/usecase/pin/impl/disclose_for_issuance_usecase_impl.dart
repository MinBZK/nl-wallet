import 'package:fimber/fimber.dart';

import '../../../../data/repository/issuance/issuance_repository.dart';
import '../../../../util/extension/core_error_extension.dart';
import '../../../../util/extension/wallet_instruction_result_extension.dart';
import '../../../../wallet_core/error/core_error.dart';
import '../../../model/result/application_error.dart';
import '../disclose_for_issuance_usecase.dart';

class DiscloseForIssuanceUseCaseImpl extends DiscloseForIssuanceUseCase {
  final IssuanceRepository _issuanceRepository;

  DiscloseForIssuanceUseCaseImpl(this._issuanceRepository);

  @override
  Future<Result<String?>> invoke(String pin) async {
    try {
      final result = await _issuanceRepository.discloseForIssuance(pin);
      return result.asApplicationResult();
    } on CoreError catch (ex) {
      Fimber.e('Failed to disclose for issuance', ex: ex);
      return Result.error(await ex.asApplicationError());
    } catch (ex) {
      Fimber.e('Failed to disclose for issuance', ex: ex);
      return Result.error(GenericError(ex.toString(), sourceError: ex));
    }
  }
}
