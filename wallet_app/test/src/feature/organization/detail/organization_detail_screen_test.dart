import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/feature/organization/detail/bloc/organization_detail_bloc.dart';
import 'package:wallet/src/feature/organization/detail/organization_detail_screen.dart';
import 'package:wallet/src/util/formatter/country_code_formatter.dart';

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
        OrganizationDetailScreen(
          onReportIssuePressed: () {},
        ).withState<OrganizationDetailBloc, OrganizationDetailState>(
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
        OrganizationDetailScreen(
          onReportIssuePressed: () {},
        ).withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          OrganizationDetailSuccess(
            organization: WalletMockData.organization,
            sharedDataWithOrganizationBefore: false,
          ),
        ),
        surfaceSize: iphoneXSizeLandscape,
      );

      await screenMatchesGolden('success.light.landscape');
    });

    testGoldens('OrganizationDetailSuccess dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        OrganizationDetailScreen(
          onReportIssuePressed: () {},
        ).withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          OrganizationDetailSuccess(
            organization: WalletMockData.organization.copyWith(supportUri: 'mailto:john.doe@example.org'),
            sharedDataWithOrganizationBefore: false,
          ),
        ),
        brightness: Brightness.dark,
        surfaceSize: const Size(375, 900), // extra tall so button is visible
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
      final title = l10n.organizationDetailScreenTitle(WalletMockData.organization.displayName);
      expect(find.text(title), findsOneWidget);
      expect(find.text(WalletMockData.organization.description!.testValue), findsOneWidget);
      expect(find.text(WalletMockData.organization.legalName), findsOneWidget);
      expect(find.text(WalletMockData.organization.type!.testValue), findsOneWidget);
      expect(find.text(WalletMockData.organization.supportUri!.replaceAll('https://', '')), findsOneWidget);
      expect(find.text(WalletMockData.organization.privacyPolicyUri!.replaceAll('https://', '')), findsOneWidget);
      expect(find.text(WalletMockData.organization.organizationId), findsOneWidget);
      final location = CountryCodeFormatter.format(WalletMockData.organization.countryCode);
      expect(location, equals('Netherlands'));
      expect(find.text(location!), findsOneWidget);
      expect(find.text(WalletMockData.organization.organizationId), findsOneWidget);
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
        surfaceSize: const Size(375, 900), // extra tall so button is visible
      );

      final l10n = await TestUtils.englishLocalizations;
      final reportIssueButtonFinder = find.text(l10n.organizationDetailScreenReportIssueCta);
      expect(reportIssueButtonFinder, findsOneWidget);
      await tester.tap(reportIssueButtonFinder);
      expect(isCalled, isTrue);
    });
  });
}
