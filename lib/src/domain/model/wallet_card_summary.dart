import 'data_highlight.dart';
import 'timeline/timeline_attribute.dart';
import 'wallet_card.dart';

class WalletCardSummary {
  final WalletCard card;
  final DataHighlight dataHighlight;
  final InteractionAttribute? interactionAttribute;

  const WalletCardSummary({
    required this.card,
    required this.dataHighlight,
    required this.interactionAttribute,
  });
}
