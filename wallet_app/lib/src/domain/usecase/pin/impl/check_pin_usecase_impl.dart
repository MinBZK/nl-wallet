import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../../util/extension/wallet_instruction_result_extension.dart';
import '../unlock_wallet_with_pin_usecase.dart';

// Checks if the provided pin matches the registered pin
class CheckPinUseCaseImpl extends CheckPinUseCase {
  final WalletRepository walletRepository;

  CheckPinUseCaseImpl(this.walletRepository);

  @override
  Future<CheckPinResult> invoke(String pin) async {
    final result = await walletRepository.checkPin(pin);
    return result.asCheckPinResult();
  }
}
