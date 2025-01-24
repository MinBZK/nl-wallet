import 'attribute/attribute.dart';
import 'organization.dart';
import 'policy/policy.dart';
import 'wallet_card.dart';

class IssuanceResponse {
  final Organization organization;
  final List<MockRequestedAttribute> requestedAttributes;
  final LocalizedText requestPurpose;
  final Policy policy;
  final List<WalletCard> cards;

  const IssuanceResponse({
    required this.organization,
    required this.requestedAttributes,
    required this.requestPurpose,
    required this.policy,
    required this.cards,
  });
}
