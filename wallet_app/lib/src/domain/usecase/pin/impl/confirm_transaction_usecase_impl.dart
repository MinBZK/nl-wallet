import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../../wallet_constants.dart';
import '../confirm_transaction_usecase.dart';

class ConfirmTransactionUseCaseImpl extends ConfirmTransactionUseCase {
  final WalletRepository walletRepository;

  ConfirmTransactionUseCaseImpl(this.walletRepository);

  @override
  Future<bool> invoke(String pin) async {
    await Future.delayed(kDefaultMockDelay);
    return walletRepository.confirmTransaction(pin);
  }
}
