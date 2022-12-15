import '../../../feature/verification/model/organization.dart';
import '../../model/wallet_card.dart';
import 'wallet_add_issued_card_usecase.dart';

/// Identical to [WalletAddIssuedCardUseCase] but allows adding multiple cards in one go.
class WalletAddIssuedCardsUseCase {
  final WalletAddIssuedCardUseCase useCase;

  WalletAddIssuedCardsUseCase(this.useCase);

  Future<void> invoke(List<WalletCard> cards, Organization organization) async {
    for (final card in cards) {
      await useCase.invoke(card, organization);
    }
  }
}
