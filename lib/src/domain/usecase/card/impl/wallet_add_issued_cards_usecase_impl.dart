import '../../../../feature/verification/model/organization.dart';
import '../../../model/wallet_card.dart';
import '../wallet_add_issued_card_usecase.dart';
import '../wallet_add_issued_cards_usecase.dart';

/// Identical to [WalletAddIssuedCardUseCase] but allows adding multiple cards in one go.
class WalletAddIssuedCardsUseCaseImpl extends WalletAddIssuedCardsUseCase {
  final WalletAddIssuedCardUseCase useCase;

  WalletAddIssuedCardsUseCaseImpl(this.useCase);

  @override
  Future<void> invoke(List<WalletCard> cards, Organization organization) async {
    for (final card in cards) {
      await useCase.invoke(card, organization);
    }
  }
}
