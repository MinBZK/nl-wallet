import '../../../feature/verification/model/organization.dart';
import '../../model/wallet_card.dart';

abstract class WalletAddIssuedCardsUseCase {
  Future<void> invoke(List<WalletCard> cards, Organization organization);
}
