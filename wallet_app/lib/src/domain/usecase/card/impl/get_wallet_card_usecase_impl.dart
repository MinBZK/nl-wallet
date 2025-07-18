import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../model/card/wallet_card.dart';
import '../../../model/result/result.dart';
import '../get_wallet_card_usecase.dart';

class GetWalletCardUseCaseImpl extends GetWalletCardUseCase {
  final WalletCardRepository _walletCardRepository;

  GetWalletCardUseCaseImpl(this._walletCardRepository);

  @override
  Future<Result<WalletCard>> invoke(String attestationId) async {
    return tryCatch(
      () async => _walletCardRepository.read(attestationId),
      'Failed to load card with id: $attestationId',
    );
  }
}
