import '../../../data/repository/wallet/wallet_repository.dart';
import '../../../wallet_constants.dart';
import 'base_check_pin_usecase.dart';

class ConfirmTransactionUseCase extends CheckPinUseCase {
  final WalletRepository walletRepository;

  ConfirmTransactionUseCase(this.walletRepository);

  @override
  Future<bool> invoke(String pin) async {
    await Future.delayed(kDefaultMockDelay);
    return walletRepository.confirmTransaction(pin);
  }
}
