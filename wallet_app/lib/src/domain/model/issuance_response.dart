import 'attribute/attribute.dart';
import 'card/wallet_card.dart';
import 'organization.dart';

class IssuanceResponse {
  final Organization organization;
  final List<MockRequestedAttribute> requestedAttributes;
  final LocalizedText requestPurpose;
  final List<WalletCard> cards;

  const IssuanceResponse({
    required this.organization,
    required this.requestedAttributes,
    required this.requestPurpose,
    required this.cards,
  });
}
