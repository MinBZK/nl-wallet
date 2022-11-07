import 'data_highlight.dart';
import 'usage_attribute.dart';
import 'wallet_card.dart';

class WalletCardSummary {
  final WalletCard card;
  final DataHighlight dataHighlight;
  final UsageAttribute usageAttribute;

  const WalletCardSummary({
    required this.card,
    required this.dataHighlight,
    required this.usageAttribute,
  });
}
