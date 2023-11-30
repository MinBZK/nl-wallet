import '../../../../data/repository/issuance/issuance_repository.dart';
import '../../../../util/extension/wallet_instruction_result_extension.dart';
import '../disclose_for_issuance_usecase.dart';

class DiscloseForIssuanceUseCaseImpl extends DiscloseForIssuanceUseCase {
  final IssuanceRepository walletRepository;

  DiscloseForIssuanceUseCaseImpl(this.walletRepository);

  @override
  Future<CheckPinResult> invoke(String pin) async {
    final result = await walletRepository.discloseForIssuance(pin);
    return result.asCheckPinResult();
  }
}
