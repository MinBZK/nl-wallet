import '../../../../../environment.dart';
import '../../../../data/repository/pid/pid_repository.dart';
import '../../../../data/repository/wallet/wallet_repository.dart';
import '../setup_mocked_wallet_usecase.dart';

class SetupMockedWalletUseCaseImpl extends SetupMockedWalletUseCase {
  final WalletRepository _walletRepository;
  final PidRepository _pidRepository;

  SetupMockedWalletUseCaseImpl(
    this._walletRepository,
    this._pidRepository,
  );

  @override
  Future<void> invoke() async {
    if (!Environment.mockRepositories) {
      throw UnsupportedError('Configuring a mocked wallet is only possible on mock builds');
    }
    // Create wallet
    await _walletRepository.createWallet('000000');

    // Add cards + history
    await _pidRepository.acceptIssuance('000000');
  }
}
