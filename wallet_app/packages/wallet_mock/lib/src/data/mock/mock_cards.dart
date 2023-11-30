import 'package:wallet_core/core.dart';

import 'mock_attributes.dart';

final kPidCards = [
  Card(
    persistence: CardPersistence.stored(id: 'pid'),
    docType: kPidDocType,
    attributes: kMockPidDataAttributes,
  ),
  Card(
    persistence: CardPersistence.stored(id: 'address'),
    docType: kAddressDocType,
    attributes: kMockAddressDataAttributes,
  ),
];
