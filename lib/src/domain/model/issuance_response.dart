import '../../feature/verification/model/organization.dart';
import 'data_attribute.dart';
import 'wallet_card.dart';

class IssuanceResponse {
  final Organization organization;
  final List<DataAttribute> requestedAttributes;
  final List<WalletCard> cards;

  const IssuanceResponse({
    required this.organization,
    required this.requestedAttributes,
    required this.cards,
  });
}
