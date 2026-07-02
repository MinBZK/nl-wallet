import 'dart:ui';

import 'package:collection/collection.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/card/format/attestation_format.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/domain/model/policy/organization_policy.dart';
import 'package:wallet/src/feature/common/screen/request_details_screen.dart';
import 'package:wallet/src/util/extension/localized_text_extension.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/util/mapper/context_mapper.dart';
import 'package:wallet/src/util/mapper/policy/policy_body_text_mapper.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

void main() {
  group('widgets', () {
    testWidgets("The event's attributes labels are shown", (tester) async {
      final event = WalletMockData.disclosureEvent;
      await tester.pumpWidgetWithAppWrapper(
        RequestDetailsScreen.forDisclosureEvent(
          'Title',
          event,
        ).withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
      );

      final allAttributes = event.cards.map((card) => card.attributes).flattened;
      final allLabels = allAttributes.map((attribute) => attribute.label.testValue);
      for (final label in allLabels) {
        expect(find.text(label), findsOneWidget);
      }
    });

    testWidgets("The event's purpose is shown", (tester) async {
      final event = WalletMockData.disclosureEvent;
      await tester.pumpWidgetWithAppWrapper(
        RequestDetailsScreen.forDisclosureEvent(
          'Title',
          event,
        ).withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
      );

      expect(find.text(event.purpose.testValue), findsOneWidget);
    });

    testWidgets('A stopped (cancelled) event renders read-only cards without the View CTA', (tester) async {
      final event = WalletMockData.cancelledDisclosureEvent;
      await tester.pumpWidgetWithAppWrapper(
        RequestDetailsScreen.forDisclosureEvent(
          'Title',
          event,
        ).withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
      );

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(event.cards.first.attributes.first.label.testValue), findsOneWidget);
      expect(find.text(l10n.sharedAttributesCardCta), findsNothing);
    });

    testWidgets('A failed (error) event keeps the interactive View CTA', (tester) async {
      final event = WalletMockData.failedDisclosureEvent;
      await tester.pumpWidgetWithAppWrapper(
        RequestDetailsScreen.forDisclosureEvent(
          'Title',
          event,
        ).withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
      );

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.sharedAttributesCardCta), findsWidgets);
    });
  });

  group('goldens', () {
    testGoldens('RequestDetails - light', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        RequestDetailsScreen.forDisclosureEvent(
          l10n.requestDetailScreenTitle,
          WalletMockData.disclosureEvent,
        ).withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
      );
      await screenMatchesGolden('request_details/light');
    });

    testGoldens('RequestDetails, event without attributes - light', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        RequestDetailsScreen.forDisclosureEvent(
          l10n.requestDetailScreenTitle,
          WalletEvent.disclosure(
                dateTime: DateTime(2024, 3, 1),
                status: EventStatus.error,
                relyingParty: WalletMockData.organization,
                purpose: 'Sample where no attributes are available'.untranslated,
                cards: [
                  WalletCard(
                    attestationId: 'id',
                    attestationType: 'com.example.attestationType',
                    format: AttestationFormat.sdJwt,
                    issuer: WalletMockData.organization,
                    status: WalletMockData.status,
                    metadata: WalletMockData.card.metadata,
                    attributes: [],
                  ),
                ],
                policy: WalletMockData.policy,
                type: DisclosureType.regular,
              )
              as DisclosureEvent,
        ).withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
      );
      await screenMatchesGolden('request_details/no_attributes.light');
    });

    testGoldens('RequestDetails, stopped (cancelled) event - light', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        RequestDetailsScreen.forDisclosureEvent(
          l10n.requestDetailScreenTitle,
          WalletMockData.cancelledDisclosureEvent,
        ).withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
      );
      await screenMatchesGolden('request_details/cancelled.light');
    });

    testGoldens('RequestDetails, stopped (cancelled) event - dark', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        RequestDetailsScreen.forDisclosureEvent(
          l10n.requestDetailScreenTitle,
          WalletMockData.cancelledDisclosureEvent,
        ).withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('request_details/cancelled.dark');
    });

    testGoldens('RequestDetails - dark, landscape', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        RequestDetailsScreen.forDisclosureEvent(
          l10n.requestDetailScreenTitle,
          WalletMockData.disclosureEvent,
        ).withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('request_details/dark.landscape');
    });
  });
}
