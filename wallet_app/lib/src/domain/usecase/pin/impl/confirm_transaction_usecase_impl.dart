import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../../wallet_constants.dart';
import '../confirm_transaction_usecase.dart';

class ConfirmTransactionUseCaseImpl extends ConfirmTransactionUseCase {
  final WalletRepository walletRepository;

  ConfirmTransactionUseCaseImpl(this.walletRepository);

  @override
  Future<CheckPinResult> invoke(String pin) async {
    await Future.delayed(kDefaultMockDelay);
    return await walletRepository.confirmTransaction(pin);
  }
}
