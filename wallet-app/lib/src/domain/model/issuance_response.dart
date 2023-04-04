import '../../feature/verification/model/organization.dart';
import 'attribute/requested_attribute.dart';
import 'policy/policy.dart';
import 'wallet_card.dart';

class IssuanceResponse {
  final Organization organization;
  final List<RequestedAttribute> requestedAttributes;
  final Policy policy;
  final List<WalletCard> cards;

  const IssuanceResponse({
    required this.organization,
    required this.requestedAttributes,
    required this.policy,
    required this.cards,
  });
}
