import '../../feature/verification/model/organization.dart';
import 'attribute/requested_attribute.dart';
import 'policy/interaction_policy.dart';
import 'wallet_card.dart';

class IssuanceResponse {
  final Organization organization;
  final List<RequestedAttribute> requestedAttributes;
  final InteractionPolicy interactionPolicy;
  final List<WalletCard> cards;

  const IssuanceResponse({
    required this.organization,
    required this.requestedAttributes,
    required this.interactionPolicy,
    required this.cards,
  });
}
