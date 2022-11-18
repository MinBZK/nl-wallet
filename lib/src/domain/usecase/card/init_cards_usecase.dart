import '../../../data/repository/card/wallet_card_repository.dart';
import '../../../data/repository/issuance/issuance_response_repository.dart';
import '../../model/issuance_response.dart';

/// Temporary usecase to make sure the app initializes with cards
class InitCardsUseCase {
  final IssuanceResponseRepository issuanceRepository;
  final WalletCardRepository walletCardRepository;

  InitCardsUseCase(this.issuanceRepository, this.walletCardRepository) {
    invoke();
  }

  void invoke() async {
    final IssuanceResponse issuanceResponse = await issuanceRepository.read('PID_1');
    walletCardRepository.create(issuanceResponse.cards.first);
  }
}
