import '../../../feature/verification/model/organization.dart';
import '../../model/wallet_card.dart';

abstract class WalletAddIssuedCardUseCase {
  Future<void> invoke(WalletCard card, Organization organization);
}
