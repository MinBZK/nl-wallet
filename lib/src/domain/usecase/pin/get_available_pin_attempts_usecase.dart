import '../../../data/repository/wallet/wallet_repository.dart';

class GetAvailablePinAttemptsUseCase {
  final WalletRepository walletRepository;

  GetAvailablePinAttemptsUseCase(this.walletRepository);

  int invoke() => walletRepository.leftoverUnlockAttempts;
}
