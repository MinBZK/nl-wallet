import 'attribute/data_attribute.dart';
import 'card_front.dart';

class WalletCard {
  final String id;
  final String issuerId;
  final CardFront front;
  final List<DataAttribute> attributes;

  const WalletCard({
    required this.id,
    required this.issuerId,
    required this.front,
    required this.attributes,
  });
}
