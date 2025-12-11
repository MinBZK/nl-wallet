import 'package:flutter/material.dart';
import 'package:wallet/src/domain/model/app_image_data.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/card/metadata/card_display_metadata.dart';
import 'package:wallet/src/domain/model/card/metadata/card_rendering.dart';
import 'package:wallet/src/domain/model/card/status/card_status.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/domain/model/disclosure/disclose_card_request.dart';
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
  static Locale testLocale = const Locale('en');

  static final DateTime validFrom = DateTime(2050, 5, 1, 17, 25);
  static final DateTime validUntil = DateTime(2025, 5, 1, 17, 25);
  static final CardStatus status = CardStatusValid(validUntil: validUntil);

  static WalletCard card = WalletCard(
    attestationId: 'id',
    attestationType: 'com.example.attestationType',
    issuer: WalletMockData.organization,
    status: WalletMockData.status,
    metadata: [
      CardDisplayMetadata(
        language: testLocale,
        name: 'Sample Card #1',
        rawSummary: 'Subtitle',
        rendering: const SimpleCardRendering(
          logo: AppAssetImage(WalletAssets.logo_card_rijksoverheid),
          textColor: DarkWalletTheme.textColor,
          bgColor: Color(0xFF35426E),
        ),
      ),
    ],
    attributes: [textDataAttribute],
  );

  static WalletCard cardWithStatus(CardStatus status) => card.copyWith(status: status);

  static List<CardStatus> cardStatusList = [
    CardStatusValidSoon(validFrom: validFrom),
    CardStatusValid(validUntil: validUntil),
    CardStatusExpiresSoon(validUntil: validUntil),
    CardStatusExpired(validUntil: validUntil),
    const CardStatusRevoked(),
    const CardStatusCorrupted(),
    const CardStatusUndetermined(),
  ];

  static WalletCard simpleRenderingCard = WalletCard(
    attestationId: 'id',
    attestationType: 'com.example.attestationType',
    issuer: WalletMockData.organization,
    status: WalletMockData.status,
    metadata: [
      CardDisplayMetadata(
        language: testLocale,
        name: 'Simple Rendering',
        description: 'Sample card with simple rendering metadata',
        rawSummary: 'Sample summary (no placeholders)',
        rendering: const SimpleCardRendering(textColor: Colors.white, bgColor: Colors.deepPurple),
      ),
    ],
    attributes: [textDataAttribute],
  );

  static WalletCard altCard = WalletCard(
    attestationId: 'id2',
    attestationType: 'com.example.alt.attestationType',
    issuer: WalletMockData.organization,
    status: WalletMockData.status,
    metadata: [
      CardDisplayMetadata(
        language: testLocale,
        name: 'Sample Card #2',
        rawSummary: 'Alt Subtitle',
        rendering: const SimpleCardRendering(
          textColor: LightWalletTheme.textColor,
          logo: AppAssetImage(WalletAssets.logo_card_rijksoverheid),
          bgColor: Color(0xFFCCEFF0),
        ),
      ),
    ],
    attributes: [textDataAttribute, textDataAttribute, textDataAttribute],
  );

  static DiscloseCardRequest discloseCardRequestSingleCard = DiscloseCardRequest.fromCard(card);
  static DiscloseCardRequest discloseCardRequestMultiCard = DiscloseCardRequest(candidates: [card, altCard]);

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
    svgId: 'text_svgId',
    label: 'Label',
    value: const StringValue('Value'),
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

  /// Disclosure events

  static DisclosureEvent get disclosureEvent =>
      WalletEvent.disclosure(
            dateTime: DateTime(2024, 3, 1),
            status: EventStatus.success,
            relyingParty: organization,
            purpose: 'disclosure'.untranslated,
            cards: [card],
            policy: policy,
            type: DisclosureType.regular,
          )
          as DisclosureEvent;

  static DisclosureEvent get loginEvent =>
      WalletEvent.disclosure(
            dateTime: DateTime(2024, 2, 1),
            status: EventStatus.success,
            relyingParty: organization,
            purpose: 'disclosure'.untranslated,
            cards: [card],
            policy: policy,
            type: DisclosureType.login,
          )
          as DisclosureEvent;

  static DisclosureEvent get failedLoginEvent =>
      WalletEvent.disclosure(
            dateTime: DateTime(2024, 2, 1),
            status: EventStatus.error,
            relyingParty: organization,
            purpose: 'disclosure'.untranslated,
            cards: [card],
            policy: policy,
            type: DisclosureType.login,
          )
          as DisclosureEvent;

  static DisclosureEvent get failedLoginEventNothingShared =>
      WalletEvent.disclosure(
            dateTime: DateTime(2024, 2, 1, 22, 11),
            status: EventStatus.error,
            relyingParty: organization,
            purpose: 'disclosure'.untranslated,
            cards: const [],
            policy: policy,
            type: DisclosureType.login,
          )
          as DisclosureEvent;

  static DisclosureEvent get failedDisclosureEvent =>
      WalletEvent.disclosure(
            dateTime: DateTime(2024, 2, 1),
            status: EventStatus.error,
            relyingParty: organization,
            purpose: 'disclosure'.untranslated,
            cards: [card],
            policy: policy,
            type: DisclosureType.regular,
          )
          as DisclosureEvent;

  static DisclosureEvent get failedDisclosureEventNothingShared =>
      WalletEvent.disclosure(
            dateTime: DateTime(2023, 5, 9, 11, 23),
            status: EventStatus.error,
            relyingParty: organization,
            purpose: 'disclosure - nothing shared error'.untranslated,
            cards: const [],
            policy: policy,
            type: DisclosureType.regular,
          )
          as DisclosureEvent;

  static DisclosureEvent get cancelledDisclosureEvent =>
      WalletEvent.disclosure(
            dateTime: DateTime(2024, 2, 1),
            status: EventStatus.cancelled,
            relyingParty: organization,
            purpose: 'disclosure'.untranslated,
            cards: [card],
            policy: policy,
            type: DisclosureType.regular,
          )
          as DisclosureEvent;

  /// Sign events

  static SignEvent get signEvent =>
      WalletEvent.sign(
            dateTime: DateTime(2024, 1, 1),
            status: EventStatus.success,
            relyingParty: organization,
            policy: policy,
            document: document,
          )
          as SignEvent;

  /// Issuance events

  static IssuanceEvent get issuanceEvent =>
      WalletEvent.issuance(
            dateTime: DateTime(2023, 12, 1),
            status: EventStatus.success,
            card: card,
            eventType: IssuanceEventType.cardIssued,
          )
          as IssuanceEvent;

  static IssuanceEvent get issuanceEventCardRenewed =>
      WalletEvent.issuance(
            dateTime: DateTime(2025, 2, 1),
            status: EventStatus.success,
            card: card,
            eventType: IssuanceEventType.cardRenewed,
          )
          as IssuanceEvent;

  static IssuanceEvent get issuanceEventCardStatusExpired =>
      WalletEvent.issuance(
            dateTime: DateTime(2025, 2, 1),
            status: EventStatus.success,
            card: cardWithStatus(CardStatusExpired(validUntil: validUntil)),
            eventType: IssuanceEventType.cardStatusExpired,
          )
          as IssuanceEvent;

  static IssuanceEvent get issuanceEventCardStatusRevoked =>
      WalletEvent.issuance(
            dateTime: DateTime(2025, 2, 1),
            status: EventStatus.success,
            card: cardWithStatus(const CardStatusRevoked()),
            eventType: IssuanceEventType.cardStatusRevoked,
          )
          as IssuanceEvent;

  static IssuanceEvent get issuanceEventCardStatusCorrupted =>
      WalletEvent.issuance(
            dateTime: DateTime(2025, 2, 1),
            status: EventStatus.success,
            card: cardWithStatus(const CardStatusCorrupted()),
            eventType: IssuanceEventType.cardStatusCorrupted,
          )
          as IssuanceEvent;
}
