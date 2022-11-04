import 'wallet_card.dart';
import 'wallet_card_data_attribute.dart';

class WalletCardSummary {
  final WalletCard card;
  final WalletCardDataAttribute data;
  final WalletCardDataAttribute usage;

  const WalletCardSummary({
    required this.card,
    required this.data,
    required this.usage,
  });
}
