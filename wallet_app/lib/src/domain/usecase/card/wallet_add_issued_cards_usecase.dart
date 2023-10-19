import '../../model/organization.dart';
import '../../model/wallet_card.dart';

export '../../model/organization.dart';

abstract class WalletAddIssuedCardsUseCase {
  Future<void> invoke(List<WalletCard> cards, Organization organization);
}
