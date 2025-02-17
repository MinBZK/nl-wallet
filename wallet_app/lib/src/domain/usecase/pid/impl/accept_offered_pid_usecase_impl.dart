import 'package:fimber/fimber.dart';

import '../../../../data/repository/pid/pid_repository.dart';
import '../../../../util/extension/core_error_extension.dart';
import '../../../../util/extension/wallet_instruction_result_extension.dart';
import '../../../../wallet_core/error/core_error.dart';
import '../../../model/result/application_error.dart';
import '../../../model/result/result.dart';
import '../accept_offered_pid_usecase.dart';

class AcceptOfferedPidUseCaseImpl extends AcceptOfferedPidUseCase {
  final PidRepository _pidRepository;

  AcceptOfferedPidUseCaseImpl(this._pidRepository);

  @override
  Future<Result<String?>> invoke(String pin) async {
    try {
      final result = await _pidRepository.acceptOfferedPid(pin);
      return result.asApplicationResult();
    } on CoreError catch (ex) {
      Fimber.e('Failed to accept pid', ex: ex);
      return Result.error(await ex.asApplicationError());
    } catch (ex) {
      Fimber.e('Failed to accept pid', ex: ex);
      return Result.error(GenericError(ex.toString(), sourceError: ex));
    }
  }
}
