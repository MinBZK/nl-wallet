import 'package:wallet_core/core.dart';

import 'mock_attributes.dart';
import 'mock_organizations.dart';

final kPidCards = [
  Card(
    persistence: CardPersistence.stored(id: 'pid'),
    docType: kPidDocType,
    attributes: kMockPidDataAttributes,
    issuer: kOrganizations[kRvigId]!,
  ),
  Card(
    persistence: CardPersistence.stored(id: 'address'),
    docType: kAddressDocType,
    attributes: kMockAddressDataAttributes,
    issuer: kOrganizations[kRvigId]!,
  ),
];
