import 'package:wallet/src/domain/model/attribute/data_attribute.dart';
import 'package:wallet/src/domain/model/card_front.dart';
import 'package:wallet/src/domain/model/document.dart';
import 'package:wallet/src/domain/model/policy/policy.dart';
import 'package:wallet/src/domain/model/wallet_card.dart';
import 'package:wallet/src/feature/verification/model/organization.dart';

abstract class WalletMockData {
  static const WalletCard card = WalletCard(
    front: cardFront,
    attributes: [imageDataAttribute, textDataAttribute],
    id: 'id',
    issuerId: 'id',
  );

  static const CardFront cardFront = CardFront(
    title: 'Sample Card',
    backgroundImage: 'assets/svg/rijks_card_bg_dark.svg',
    theme: CardFrontTheme.dark,
    info: 'Info',
    logoImage: 'assets/non-free/images/logo_card_rijksoverheid.png',
    subtitle: 'Subtitle',
  );

  static const Organization organization = Organization(
    id: 'id',
    name: 'Organization Name',
    category: 'Category',
    shortName: 'This is a TimelineAttributeRow',
    description: 'Organization description',
    logoUrl: 'assets/non-free/images/logo_rijksoverheid.png',
  );

  static const DataAttribute textDataAttribute = DataAttribute(
    label: 'Label',
    value: 'Value',
    sourceCardId: '',
    valueType: AttributeValueType.text,
  );

  static const DataAttribute imageDataAttribute = DataAttribute(
    label: 'Label',
    value: 'assets/non-free/images/person_x.png',
    sourceCardId: '',
    valueType: AttributeValueType.image,
  );

  static const Policy policy = Policy(
    storageDuration: Duration(days: 90),
    dataPurpose: 'Data Purpose',
    dataIsShared: false,
    dataIsSignature: false,
    dataContainsSingleViewProfilePhoto: false,
    deletionCanBeRequested: true,
    privacyPolicyUrl: 'https://www.example.org',
  );

  static const Document document = Document(
    title: 'Title',
    fileName: 'docs/agreement.pdf',
    url: 'https://example.org/agreement.pdf',
  );
}
