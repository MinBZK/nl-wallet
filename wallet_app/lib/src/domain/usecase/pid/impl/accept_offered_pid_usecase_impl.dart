import '../../../../data/repository/pid/pid_repository.dart';
import '../../../../util/extension/wallet_instruction_result_extension.dart';
import '../accept_offered_pid_usecase.dart';

class AcceptOfferedPidUseCaseImpl implements AcceptOfferedPidUseCase {
  final PidRepository _pidRepository;

  const AcceptOfferedPidUseCaseImpl(this._pidRepository);

  @override
  Future<CheckPinResult> invoke(String pin) async {
    final result = await _pidRepository.acceptOfferedPid(pin);
    return result.asCheckPinResult();
  }
}
