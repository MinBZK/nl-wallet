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
    final IssuanceResponse passport = await issuanceRepository.read('1');
    final IssuanceResponse drivingLicense = await issuanceRepository.read('2');
    walletCardRepository.create(passport.cards.first);
    walletCardRepository.create(drivingLicense.cards.first);
  }
}
