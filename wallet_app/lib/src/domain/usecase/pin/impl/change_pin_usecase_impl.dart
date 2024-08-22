import '../../../../data/repository/wallet/wallet_repository.dart';
import '../change_pin_usecase.dart';

class ChangePinUseCaseImpl extends ChangePinUseCase {
  final WalletRepository walletRepository;

  ChangePinUseCaseImpl(this.walletRepository);

  @override
  Future<void> invoke(String oldPin, String newPin) => walletRepository.changePin(oldPin, newPin);
}
