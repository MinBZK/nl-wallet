import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/disclosure/disclose_card_request.dart';
import 'package:wallet/src/domain/model/policy/organization_policy.dart';
import 'package:wallet/src/feature/disclosure/page/disclosure_confirm_data_attributes_page.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/util/mapper/context_mapper.dart';
import 'package:wallet/src/util/mapper/policy/policy_body_text_mapper.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../test_util/test_utils.dart';

void main() {
  testWidgets('card titles are shown', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      Provider<ContextMapper<OrganizationPolicy, String>>(
        create: (c) => PolicyBodyTextMapper(),
        child: DisclosureConfirmDataAttributesPage(
          onAcceptPressed: () {},
          onDeclinePressed: () {},
          onAlternativeCardSelected: (update) {},
          relyingParty: WalletMockData.organization,
          cardRequests: [
            DiscloseCardRequest.fromCard(WalletMockData.card),
            DiscloseCardRequest.fromCard(WalletMockData.altCard),
          ],
          policy: WalletMockData.policy,
          requestPurpose: 'data purpose'.untranslated,
        ),
      ),
    );

    // Check if the card title is shown
    final cardFinder = find.textContaining(WalletMockData.card.title.testValue);
    expect(cardFinder, findsOneWidget);
    // Check if the altCard title is shown
    final altCardFinder = find.textContaining(WalletMockData.altCard.title.testValue);
    expect(altCardFinder, findsOneWidget);
  });

  testWidgets('organization title is shown', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      Provider<ContextMapper<OrganizationPolicy, String>>(
        create: (c) => PolicyBodyTextMapper(),
        child: DisclosureConfirmDataAttributesPage(
          onAcceptPressed: () {},
          onDeclinePressed: () {},
          onAlternativeCardSelected: (update) {},
          relyingParty: WalletMockData.organization,
          cardRequests: [DiscloseCardRequest.fromCard(WalletMockData.card)],
          policy: WalletMockData.policy,
          requestPurpose: 'data purpose'.untranslated,
        ),
      ),
    );

    // Organization name should be shown 3 times on this screen:
    // - title
    // - organization detail row
    // - agreements section
    final orgNameFinder = find.textContaining(WalletMockData.organization.displayName.testValue);
    expect(orgNameFinder, findsNWidgets(3));
  });

  testWidgets('data purpose is shown', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      Provider<ContextMapper<OrganizationPolicy, String>>(
        create: (c) => PolicyBodyTextMapper(),
        child: DisclosureConfirmDataAttributesPage(
          onAcceptPressed: () {},
          onDeclinePressed: () {},
          onAlternativeCardSelected: (update) {},
          relyingParty: WalletMockData.organization,
          cardRequests: [DiscloseCardRequest.fromCard(WalletMockData.card)],
          policy: WalletMockData.policy,
          requestPurpose: 'data purpose'.untranslated,
        ),
      ),
    );

    // Check if the purpose is shown
    final titleFinder = find.text('data purpose');
    expect(titleFinder, findsOneWidget);
  });

  testWidgets('verify decline button callback', (tester) async {
    bool isCalled = false;
    await tester.pumpWidgetWithAppWrapper(
      Provider<ContextMapper<OrganizationPolicy, String>>(
        create: (c) => PolicyBodyTextMapper(),
        child: DisclosureConfirmDataAttributesPage(
          onAcceptPressed: () {},
          onDeclinePressed: () => isCalled = true,
          onAlternativeCardSelected: (update) {},
          relyingParty: WalletMockData.organization,
          cardRequests: [DiscloseCardRequest.fromCard(WalletMockData.card)],
          policy: WalletMockData.policy,
          requestPurpose: 'data purpose'.untranslated,
        ),
      ),
    );

    // Check if the deny listener is triggered correctly
    final l10n = await TestUtils.englishLocalizations;
    final declineButtonFinder = find.text(l10n.disclosureConfirmDataAttributesPageDenyCta, skipOffstage: false);
    // Scroll the button into the viewport so it can be tapped
    await tester.scrollUntilVisible(declineButtonFinder, 50);
    await tester.pumpAndSettle();
    await tester.tap(declineButtonFinder);
    expect(isCalled, isTrue);
  });

  testWidgets('verify accept button callback', (tester) async {
    bool isCalled = false;
    await tester.pumpWidgetWithAppWrapper(
      Provider<ContextMapper<OrganizationPolicy, String>>(
        create: (c) => PolicyBodyTextMapper(),
        child: DisclosureConfirmDataAttributesPage(
          onAcceptPressed: () => isCalled = true,
          onDeclinePressed: () {},
          onAlternativeCardSelected: (update) {},
          relyingParty: WalletMockData.organization,
          cardRequests: [DiscloseCardRequest.fromCard(WalletMockData.card)],
          policy: WalletMockData.policy,
          requestPurpose: 'data purpose'.untranslated,
        ),
      ),
    );

    // Check if accept listener is triggered correctly
    final l10n = await TestUtils.englishLocalizations;
    final acceptButtonFinder = find.text(l10n.disclosureConfirmDataAttributesPageApproveCta, skipOffstage: false);
    // Scroll the button into the viewport so it can be tapped
    await tester.scrollUntilVisible(acceptButtonFinder, 50);
    await tester.pumpAndSettle();
    await tester.tap(acceptButtonFinder);
    expect(isCalled, isTrue);
  });
}
