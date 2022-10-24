import '../../../data/repository/wallet/wallet_repository.dart';

class CheckIsAppInitializedUseCase {
  final WalletRepository walletRepository;

  CheckIsAppInitializedUseCase(this.walletRepository);

  Future<bool> isInitialized() async {
    return walletRepository.isWalletInitialized();
  }
}
