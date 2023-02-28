import '../../../../data/repository/wallet/wallet_repository.dart';
import '../get_available_pin_attempts_usecase.dart';

class GetAvailablePinAttemptsUseCaseImpl implements GetAvailablePinAttemptsUseCase {
  final WalletRepository walletRepository;

  GetAvailablePinAttemptsUseCaseImpl(this.walletRepository);

  @override
  int invoke() => walletRepository.leftoverPinAttempts;
}
