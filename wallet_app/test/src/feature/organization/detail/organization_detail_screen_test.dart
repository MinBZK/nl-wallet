import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/feature/organization/detail/bloc/organization_detail_bloc.dart';
import 'package:wallet/src/feature/organization/detail/organization_detail_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

class MockOrganizationDetailBloc extends MockBloc<OrganizationDetailEvent, OrganizationDetailState>
    implements OrganizationDetailBloc {}

void main() {
  group('goldens', () {
    testGoldens('OrganizationDetailSuccess light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        OrganizationDetailScreen(onReportIssuePressed: () {})
            .withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          OrganizationDetailSuccess(
            organization: WalletMockData.organization,
            sharedDataWithOrganizationBefore: false,
          ),
        ),
      );

      await screenMatchesGolden('success.light');
    });

    testGoldens('OrganizationDetailSuccess light - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        OrganizationDetailScreen(onReportIssuePressed: () {})
            .withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          OrganizationDetailSuccess(
            organization: WalletMockData.organization,
            sharedDataWithOrganizationBefore: false,
          ),
        ),
        surfaceSize: const Size(375, 812).flipped,
      );

      await screenMatchesGolden('success.light.landscape');
    });

    testGoldens('OrganizationDetailSuccess dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        OrganizationDetailScreen(onReportIssuePressed: () {})
            .withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          OrganizationDetailSuccess(
            organization: WalletMockData.organization,
            sharedDataWithOrganizationBefore: false,
          ),
        ),
        brightness: Brightness.dark,
      );

      await screenMatchesGolden('success.dark');
    });

    testGoldens('OrganizationDetailInitial light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const OrganizationDetailScreen().withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          OrganizationDetailInitial(),
        ),
      );
      await screenMatchesGolden('loading.light');
    });

    testGoldens('OrganizationDetailFailure light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const OrganizationDetailScreen().withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          const OrganizationDetailFailure(organizationId: 'id'),
        ),
      );
      await screenMatchesGolden('error.light');
    });
  });

  group('widgets', () {
    testWidgets('organization details are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const OrganizationDetailScreen().withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          OrganizationDetailSuccess(
            organization: WalletMockData.organization,
            sharedDataWithOrganizationBefore: false,
          ),
        ),
      );

      final l10n = await TestUtils.englishLocalizations;
      final title = l10n.organizationDetailScreenTitle(WalletMockData.organization.displayName.testValue);
      expect(find.text(title), findsNWidgets(2) /* expanded and collapsed title */);
      expect(find.text(WalletMockData.organization.description!.testValue), findsOneWidget);
      expect(find.text(WalletMockData.organization.legalName.testValue), findsOneWidget);
      expect(find.text(WalletMockData.organization.category!.testValue), findsOneWidget);
      expect(find.text(WalletMockData.organization.privacyPolicyUrl.toString()), findsOneWidget);
      expect(find.text(WalletMockData.organization.city!.testValue), findsOneWidget);
      expect(find.text(WalletMockData.organization.kvk.toString()), findsOneWidget);
      expect(find.text(l10n.organizationDetailScreenWebsiteInfo), findsNothing);
    });

    testWidgets('onReportIssuePressed callback is triggered when button is clicked', (tester) async {
      bool isCalled = false;
      await tester.pumpWidgetWithAppWrapper(
        OrganizationDetailScreen(
          onReportIssuePressed: () => isCalled = true,
        ).withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          OrganizationDetailSuccess(
            organization: WalletMockData.organization,
            sharedDataWithOrganizationBefore: false,
          ),
        ),
      );

      final l10n = await TestUtils.englishLocalizations;
      final reportIssueButtonFinder = find.text(l10n.organizationDetailScreenReportIssueCta);
      expect(reportIssueButtonFinder, findsOneWidget);
      await tester.tap(reportIssueButtonFinder);
      expect(isCalled, isTrue);
    });
  });
}
