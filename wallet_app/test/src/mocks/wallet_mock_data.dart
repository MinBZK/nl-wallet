import 'package:flutter/material.dart';
import 'package:wallet/src/domain/model/app_image_data.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/card/card_front.dart';
import 'package:wallet/src/domain/model/card/metadata/card_display_metadata.dart';
import 'package:wallet/src/domain/model/card/metadata/card_rendering.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/domain/model/document.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/domain/model/organization.dart';
import 'package:wallet/src/domain/model/policy/policy.dart';
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

  static WalletCard simpleRenderingCard = WalletCard(
    docType: 'com.example.docType',
    front: null,
    issuer: WalletMockData.organization,
    attributes: [textDataAttribute],
    metadata: const [
      CardDisplayMetadata(
        language: Locale('en'),
        name: 'Simple Rendering',
        description: 'Sample card with simple rendering metadata',
        rendering: SimpleCardRendering(textColor: Colors.white, bgColor: Colors.deepPurple),
      ),
    ],
    id: 'id',
  );

  static WalletCard altCard = WalletCard(
    front: altCardFront,
    issuer: WalletMockData.organization,
    docType: 'com.example.alt.docType',
    attributes: [textDataAttribute, textDataAttribute, textDataAttribute],
    id: 'id2',
  );

  static const CardFront cardFront = CardFront(
    title: {'': 'Sample Card #1'},
    backgroundImage: WalletAssets.svg_rijks_card_bg_dark,
    theme: CardFrontTheme.dark,
    info: {'': 'Info'},
    logoImage: WalletAssets.logo_card_rijksoverheid,
    subtitle: {'': 'Subtitle'},
  );

  static const CardFront altCardFront = CardFront(
    title: {'': 'Sample Card #2'},
    backgroundImage: WalletAssets.svg_rijks_card_bg_light,
    theme: CardFrontTheme.light,
    info: {'': 'Alt info'},
    logoImage: WalletAssets.logo_card_rijksoverheid,
    subtitle: {'': 'Alt Subtitle'},
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
    privacyPolicyUrl: 'https://example.org/privacy',
    city: 'Den Haag'.untranslated,
    department: 'department abc'.untranslated,
    kvk: '12345678',
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
        dateTime: DateTime(2024, 3, 1),
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

  static DisclosureEvent get failedDisclosureEvent => WalletEvent.disclosure(
        dateTime: DateTime(2024, 2, 1),
        status: EventStatus.error,
        relyingParty: organization,
        purpose: 'disclosure'.untranslated,
        cards: [card],
        policy: policy,
        type: DisclosureType.regular,
      ) as DisclosureEvent;

  static DisclosureEvent get cancelledDisclosureEvent => WalletEvent.disclosure(
        dateTime: DateTime(2024, 2, 1),
        status: EventStatus.cancelled,
        relyingParty: organization,
        purpose: 'disclosure'.untranslated,
        cards: [card],
        policy: policy,
        type: DisclosureType.regular,
      ) as DisclosureEvent;
}
