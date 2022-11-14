import '../../feature/verification/model/organization.dart';
import 'wallet_card.dart';

class IssueResponse {
  final Organization organization;
  final List<WalletCard> cards;

  const IssueResponse({
    required this.organization,
    required this.cards,
  });
}
