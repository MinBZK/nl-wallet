import '../../../data/repository/card/wallet_card_repository.dart';
import '../../../data/repository/issuer/issue_response_repository.dart';
import '../../model/issue_response.dart';

/// Temporary usecase to make sure the app initializes with cards
class InitCardsUseCase {
  final IssueResponseRepository issueResponseRepository;
  final WalletCardRepository walletCardRepository;

  InitCardsUseCase(this.issueResponseRepository, this.walletCardRepository) {
    invoke();
  }

  void invoke() async {
    final IssueResponse passport = await issueResponseRepository.read('1');
    final IssueResponse drivingLicense = await issueResponseRepository.read('2');
    walletCardRepository.create(passport.cards.first);
    walletCardRepository.create(drivingLicense.cards.first);
  }
}
