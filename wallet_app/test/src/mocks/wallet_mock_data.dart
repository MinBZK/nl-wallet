import 'package:flutter/material.dart';
import 'package:wallet/src/domain/model/app_image_data.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/card/metadata/card_display_metadata.dart';
import 'package:wallet/src/domain/model/card/metadata/card_rendering.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/domain/model/document.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/domain/model/organization.dart';
import 'package:wallet/src/domain/model/policy/policy.dart';
import 'package:wallet/src/domain/model/wallet_card_detail.dart';
import 'package:wallet/src/theme/dark_wallet_theme.dart';
import 'package:wallet/src/theme/light_wallet_theme.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/wallet_assets.dart';

abstract class WalletMockData {
  static Locale testLocale = Locale('en');

  static WalletCard card = WalletCard(
    docType: 'com.example.docType',
    issuer: WalletMockData.organization,
    attributes: [textDataAttribute],
    id: 'id',
    metadata: [
      CardDisplayMetadata(
        language: testLocale,
        name: 'Sample Card #1',
        rawSummary: 'Subtitle',
        rendering: SimpleCardRendering(
          logoUri: WalletAssets.logo_card_rijksoverheid,
          textColor: DarkWalletTheme.textColor,
          bgColor: Color(0xFF35426E),
        ),
      ),
    ],
  );

  static WalletCard simpleRenderingCard = WalletCard(
    docType: 'com.example.docType',
    issuer: WalletMockData.organization,
    attributes: [textDataAttribute],
    metadata: [
      CardDisplayMetadata(
        language: testLocale,
        name: 'Simple Rendering',
        description: 'Sample card with simple rendering metadata',
        rawSummary: 'Sample summary (no placeholders)',
        rendering: SimpleCardRendering(textColor: Colors.white, bgColor: Colors.deepPurple),
      ),
    ],
    id: 'id',
  );

  static WalletCard altCard = WalletCard(
    issuer: WalletMockData.organization,
    docType: 'com.example.alt.docType',
    attributes: [textDataAttribute, textDataAttribute, textDataAttribute],
    id: 'id2',
    metadata: [
      CardDisplayMetadata(
        language: testLocale,
        name: 'Sample Card #2',
        rawSummary: 'Alt Subtitle',
        rendering: SimpleCardRendering(
          textColor: LightWalletTheme.textColor,
          logoUri: WalletAssets.logo_card_rijksoverheid,
          bgColor: Color(0xFFCCEFF0),
        ),
      ),
    ],
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

  static DisclosureEvent get failedLoginEvent => WalletEvent.disclosure(
        dateTime: DateTime(2024, 2, 1),
        status: EventStatus.error,
        relyingParty: organization,
        purpose: 'disclosure'.untranslated,
        cards: [card],
        policy: policy,
        type: DisclosureType.login,
      ) as DisclosureEvent;

  static DisclosureEvent get failedLoginEventNothingShared => WalletEvent.disclosure(
        dateTime: DateTime(2024, 2, 1, 22, 11),
        status: EventStatus.error,
        relyingParty: organization,
        purpose: 'disclosure'.untranslated,
        cards: const [],
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

  static DisclosureEvent get failedDisclosureEventNothingShared => WalletEvent.disclosure(
        dateTime: DateTime(2023, 5, 9, 11, 23),
        status: EventStatus.error,
        relyingParty: organization,
        purpose: 'disclosure - nothing shared error'.untranslated,
        cards: const [],
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
