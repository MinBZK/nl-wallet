import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../../util/extension/wallet_instruction_result_extension.dart';
import '../unlock_wallet_with_pin_usecase.dart';

class UnlockWalletWithPinUseCaseImpl extends UnlockWalletWithPinUseCase {
  final WalletRepository walletRepository;

  UnlockWalletWithPinUseCaseImpl(this.walletRepository);

  @override
  Future<CheckPinResult> invoke(String pin) async {
    final result = await walletRepository.unlockWallet(pin);
    return result.asCheckPinResult();
  }
}
