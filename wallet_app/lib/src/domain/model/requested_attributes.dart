import 'package:collection/collection.dart';

import 'attribute/attribute.dart';
import 'card/wallet_card.dart';

typedef RequestedAttributes = Map<WalletCard, List<DataAttribute>>;

extension RequestedAttributesExtension on RequestedAttributes {
  /// The cards tied to the RequestedAttributes
  List<WalletCard> get cards => keys.toList();

  /// The attributes that are being requested
  List<DataAttribute> get attributes => values.flattenedToList;
}
