import 'package:wallet/src/domain/model/app_image_data.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/card_front.dart';
import 'package:wallet/src/domain/model/document.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/domain/model/organization.dart';
import 'package:wallet/src/domain/model/policy/policy.dart';
import 'package:wallet/src/domain/model/wallet_card.dart';
import 'package:wallet/src/domain/model/wallet_card_detail.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/wallet_assets.dart';

abstract class WalletMockData {
  static WalletCard card = WalletCard(
    docType: 'com.example.docType',
    front: cardFront,
    issuer: WalletMockData.organization,
    attributes: [textDataAttribute],
    id: 'id',
  );

  static WalletCard altCard = WalletCard(
    front: CardFront(
      title: 'Sample Card #2'.untranslated,
      backgroundImage: WalletAssets.svg_rijks_card_bg_light,
      theme: CardFrontTheme.light,
      info: 'Alt Info'.untranslated,
      logoImage: WalletAssets.logo_card_rijksoverheid,
      subtitle: 'Alt Subtitle'.untranslated,
    ),
    issuer: WalletMockData.organization,
    docType: 'com.example.alt.docType',
    attributes: [textDataAttribute, textDataAttribute, textDataAttribute],
    id: 'id2',
  );

  static CardFront cardFront = CardFront(
    title: 'Sample Card #1'.untranslated,
    backgroundImage: WalletAssets.svg_rijks_card_bg_dark,
    theme: CardFrontTheme.dark,
    info: 'Info'.untranslated,
    logoImage: WalletAssets.logo_card_rijksoverheid,
    subtitle: 'Subtitle'.untranslated,
  );

  static final WalletCardDetail cardDetail = WalletCardDetail(
    card: card,
    mostRecentIssuance: null,
    mostRecentSuccessfulDisclosure: null,
  );

  static final Organization organization = Organization(
    id: 'id',
    legalName: 'Organization Legal Name'.untranslated,
    displayName: 'Organization Display Name'.untranslated,
    category: 'Category'.untranslated,
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
    deletionCanBeRequested: true,
    privacyPolicyUrl: 'https://www.example.org',
  );

  static const Document document = Document(
    title: 'Title',
    fileName: 'docs/agreement.pdf',
    url: 'https://example.org/agreement.pdf',
  );

  static DisclosureEvent get disclosureEvent => WalletEvent.disclosure(
        dateTime: DateTime(2024, 2, 1),
        status: EventStatus.success,
        relyingParty: organization,
        purpose: 'disclosure'.untranslated,
        cards: [card],
        policy: policy,
        type: DisclosureType.regular,
      ) as DisclosureEvent;

  static DisclosureEvent get loginEvent => WalletEvent.disclosure(
        dateTime: DateTime(2024, 2, 1),
        status: EventStatus.success,
        relyingParty: organization,
        purpose: 'disclosure'.untranslated,
        cards: [card],
        policy: policy,
        type: DisclosureType.login,
      ) as DisclosureEvent;

  static SignEvent get signEvent => WalletEvent.sign(
        dateTime: DateTime(2024, 1, 1),
        status: EventStatus.success,
        relyingParty: organization,
        policy: policy,
        document: document,
      ) as SignEvent;

  static IssuanceEvent get issuanceEvent => WalletEvent.issuance(
        dateTime: DateTime(2023, 12, 1),
        status: EventStatus.success,
        card: card,
      ) as IssuanceEvent;
}
