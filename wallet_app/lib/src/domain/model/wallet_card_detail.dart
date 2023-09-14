import '../../feature/verification/model/organization.dart';
import 'timeline/interaction_timeline_attribute.dart';
import 'timeline/operation_timeline_attribute.dart';
import 'wallet_card.dart';

/// Temporary model to be used in the card detail screen
class WalletCardDetail {
  final WalletCard card;
  final Organization issuer;
  final OperationTimelineAttribute? latestIssuedOperation;
  final InteractionTimelineAttribute? latestSuccessInteraction;

  const WalletCardDetail({
    required this.card,
    required this.issuer,
    required this.latestIssuedOperation,
    required this.latestSuccessInteraction,
  });
}
