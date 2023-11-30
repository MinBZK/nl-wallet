import 'package:wallet/src/domain/model/app_image_data.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/attribute/data_attribute.dart';
import 'package:wallet/src/domain/model/card_front.dart';
import 'package:wallet/src/domain/model/document.dart';
import 'package:wallet/src/domain/model/organization.dart';
import 'package:wallet/src/domain/model/policy/policy.dart';
import 'package:wallet/src/domain/model/timeline/interaction_timeline_attribute.dart';
import 'package:wallet/src/domain/model/timeline/operation_timeline_attribute.dart';
import 'package:wallet/src/domain/model/timeline/signing_timeline_attribute.dart';
import 'package:wallet/src/domain/model/wallet_card.dart';
import 'package:wallet/src/domain/model/wallet_card_detail.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/wallet_assets.dart';

abstract class WalletMockData {
  static WalletCard card = WalletCard(
    docType: 'com.example.docType',
    front: cardFront,
    attributes: [textDataAttribute],
    id: 'id',
    issuerId: 'id',
  );

  static WalletCard altCard = WalletCard(
    front: CardFront(
      title: 'Alt Sample Card'.untranslated,
      backgroundImage: WalletAssets.svg_rijks_card_bg_light,
      theme: CardFrontTheme.light,
      info: 'Alt Info'.untranslated,
      logoImage: WalletAssets.logo_card_rijksoverheid,
      subtitle: 'Alt Subtitle'.untranslated,
    ),
    docType: 'com.example.alt.docType',
    attributes: [textDataAttribute, textDataAttribute, textDataAttribute],
    id: 'id2',
    issuerId: 'id2',
  );

  static CardFront cardFront = CardFront(
    title: 'Sample Card'.untranslated,
    backgroundImage: WalletAssets.svg_rijks_card_bg_dark,
    theme: CardFrontTheme.dark,
    info: 'Info'.untranslated,
    logoImage: WalletAssets.logo_card_rijksoverheid,
    subtitle: 'Subtitle'.untranslated,
  );

  static final WalletCardDetail cardDetail = WalletCardDetail(
    card: card,
    issuer: organization,
    latestIssuedOperation: null,
    latestSuccessInteraction: null,
  );

  static final Organization organization = Organization(
    id: 'id',
    legalName: 'Organization Name'.untranslated,
    type: 'Category'.untranslated,
    displayName: 'Name'.untranslated,
    description: 'Organization description'.untranslated,
    logo: const AppAssetImage(WalletAssets.logo_rijksoverheid),
  );

  static final DataAttribute textDataAttribute = DataAttribute.untranslated(
    key: 'text_key',
    label: 'Label',
    value: const StringValue('Value'),
    sourceCardDocType: 'com.example.docType',
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
        dataAttributes: [WalletMockData.textDataAttribute],
        dateTime: DateTime(2023, 1, 1),
        organization: WalletMockData.organization,
        status: InteractionStatus.success,
        requestPurpose: 'Purpose'.untranslated,
        policy: WalletMockData.policy,
      );

  static SigningTimelineAttribute get signingTimelineAttribute => SigningTimelineAttribute(
        dataAttributes: [WalletMockData.textDataAttribute],
        dateTime: DateTime(2023, 1, 1),
        organization: WalletMockData.organization,
        status: SigningStatus.success,
        policy: WalletMockData.policy,
        document: WalletMockData.document,
      );

  static OperationTimelineAttribute get operationTimelineAttribute => OperationTimelineAttribute(
        dataAttributes: [WalletMockData.textDataAttribute],
        dateTime: DateTime(2023, 1, 1),
        organization: WalletMockData.organization,
        status: OperationStatus.issued,
        cardTitle: cardFront.title,
      );
}
