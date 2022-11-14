import '../../feature/verification/model/organization.dart';
import 'wallet_card.dart';

class IssuanceResponse {
  final Organization organization;
  final List<WalletCard> cards;

  const IssuanceResponse({
    required this.organization,
    required this.cards,
  });
}
