import 'timeline/interaction_timeline_attribute.dart';
import 'timeline/operation_timeline_attribute.dart';
import 'wallet_card.dart';

/// Temporary model to be used in the card detail screen
class WalletCardDetail {
  final WalletCard card;
  final OperationTimelineAttribute? latestIssuedOperation;
  final InteractionTimelineAttribute? latestSuccessInteraction;

  const WalletCardDetail({
    required this.card,
    required this.latestIssuedOperation,
    required this.latestSuccessInteraction,
  });
}
