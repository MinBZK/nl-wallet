import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../model/result/result.dart';
import '../delete_wallet_card_usecase.dart';

class DeleteWalletCardUseCaseImpl extends DeleteWalletCardUseCase {
  final WalletCardRepository _walletCardRepository;
  final String attestationId;

  DeleteWalletCardUseCaseImpl(this._walletCardRepository, this.attestationId);

  @override
  Future<Result<void>> invoke(String pin) async {
    return tryCatch(
      () => _walletCardRepository.delete(pin, attestationId),
      'Failed to delete card',
    );
  }
}
