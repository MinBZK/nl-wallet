import 'package:wallet/src/domain/model/attribute/data_attribute.dart';
import 'package:wallet/src/domain/model/card_front.dart';
import 'package:wallet/src/domain/model/document.dart';
import 'package:wallet/src/domain/model/policy/policy.dart';
import 'package:wallet/src/domain/model/timeline/interaction_timeline_attribute.dart';
import 'package:wallet/src/domain/model/timeline/operation_timeline_attribute.dart';
import 'package:wallet/src/domain/model/timeline/signing_timeline_attribute.dart';
import 'package:wallet/src/domain/model/wallet_card.dart';
import 'package:wallet/src/feature/verification/model/organization.dart';
import 'package:wallet/src/wallet_assets.dart';

abstract class WalletMockData {
  static const WalletCard card = WalletCard(
    front: cardFront,
    attributes: [imageDataAttribute, textDataAttribute],
    id: 'id',
    issuerId: 'id',
  );

  static const WalletCard altCard = WalletCard(
    front: CardFront(
      title: 'Alt Sample Card',
      backgroundImage: WalletAssets.svg_rijks_card_bg_light,
      theme: CardFrontTheme.light,
      info: 'Alt Info',
      logoImage: WalletAssets.logo_card_rijksoverheid,
      subtitle: 'Alt Subtitle',
    ),
    attributes: [imageDataAttribute, textDataAttribute],
    id: 'id2',
    issuerId: 'id2',
  );

  static const CardFront cardFront = CardFront(
    title: 'Sample Card',
    backgroundImage: WalletAssets.svg_rijks_card_bg_dark,
    theme: CardFrontTheme.dark,
    info: 'Info',
    logoImage: WalletAssets.logo_card_rijksoverheid,
    subtitle: 'Subtitle',
  );

  static const Organization organization = Organization(
    id: 'id',
    name: 'Organization Name',
    category: 'Category',
    shortName: 'Name',
    description: 'Organization description',
    logoUrl: WalletAssets.logo_rijksoverheid,
  );

  static const DataAttribute textDataAttribute = DataAttribute(
    key: 'text_key',
    label: 'Label',
    value: 'Value',
    sourceCardId: 'id',
    valueType: AttributeValueType.text,
  );

  static const DataAttribute imageDataAttribute = DataAttribute(
    key: 'image_key',
    label: 'Label',
    value: WalletAssets.image_person_x,
    sourceCardId: 'id',
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

  static InteractionTimelineAttribute get interactionTimelineAttribute => InteractionTimelineAttribute(
        dataAttributes: const [WalletMockData.textDataAttribute],
        dateTime: DateTime(2023, 1, 1),
        organization: WalletMockData.organization,
        status: InteractionStatus.success,
        requestPurpose: 'Purpose',
        policy: WalletMockData.policy,
      );

  static SigningTimelineAttribute get signingTimelineAttribute => SigningTimelineAttribute(
        dataAttributes: const [WalletMockData.textDataAttribute],
        dateTime: DateTime(2023, 1, 1),
        organization: WalletMockData.organization,
        status: SigningStatus.success,
        policy: WalletMockData.policy,
        document: WalletMockData.document,
      );

  static OperationTimelineAttribute get operationTimelineAttribute => OperationTimelineAttribute(
        dataAttributes: const [WalletMockData.textDataAttribute],
        dateTime: DateTime(2023, 1, 1),
        organization: WalletMockData.organization,
        status: OperationStatus.issued,
        cardTitle: cardFront.title,
      );
}
