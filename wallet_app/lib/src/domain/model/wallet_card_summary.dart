import '../../feature/verification/model/organization.dart';
import 'timeline/interaction_timeline_attribute.dart';
import 'timeline/operation_timeline_attribute.dart';
import 'wallet_card.dart';

class WalletCardSummary {
  final WalletCard card;
  final Organization issuer;
  final OperationTimelineAttribute? latestIssuedOperation;
  final InteractionTimelineAttribute? latestSuccessInteraction;

  const WalletCardSummary({
    required this.card,
    required this.issuer,
    required this.latestIssuedOperation,
    required this.latestSuccessInteraction,
  });
}
