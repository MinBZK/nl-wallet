import 'event/wallet_event.dart';
import 'wallet_card.dart';

class WalletCardDetail {
  final WalletCard card;
  final IssuanceEvent? mostRecentIssuance;
  final DisclosureEvent? mostRecentSuccessfulDisclosure;

  const WalletCardDetail({
    required this.card,
    required this.mostRecentIssuance,
    required this.mostRecentSuccessfulDisclosure,
  });
}
