import 'package:wallet_core/core.dart';

abstract class CoreMockData {
  static const Card card = Card(
    persistence: CardPersistence.stored(id: '0'),
    docType: 'docType',
    attributes: [cardAttributeName, cardAttributeCity],
    issuer: organization,
  );

  static const CardAttribute cardAttributeName = CardAttribute(
    key: 'name',
    labels: [],
    value: CardValue_String(value: 'Willeke'),
  );

  static const CardAttribute cardAttributeCity = CardAttribute(
    key: 'city',
    labels: [],
    value: CardValue_String(value: 'Den Haag'),
  );

  static const Organization organization = Organization(
    legalName: [LocalizedString(language: 'en', value: 'legalName')],
    displayName: [LocalizedString(language: 'en', value: 'displayName')],
    description: [LocalizedString(language: 'en', value: 'description')],
    category: [LocalizedString(language: 'en', value: 'category')],
  );
}
