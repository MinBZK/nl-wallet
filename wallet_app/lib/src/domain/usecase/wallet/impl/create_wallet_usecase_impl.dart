import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../model/result/result.dart';
import '../create_wallet_usecase.dart';

class CreateWalletUseCaseImpl extends CreateWalletUseCase {
  final WalletRepository _walletRepository;

  CreateWalletUseCaseImpl(this._walletRepository);

  @override
  Future<Result<void>> invoke(String pin) async {
    return tryCatch(
      () async => _walletRepository.createWallet(pin),
      'Unable to create wallet',
    );
  }
}
