import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/organization.dart';
import 'package:wallet/src/domain/model/person_type.dart';
import 'package:wallet/src/feature/organization/detail/bloc/organization_detail_bloc.dart';
import 'package:wallet/src/feature/organization/detail/organization_detail_screen.dart';
import 'package:wallet/src/util/formatter/country_code_formatter.dart';
import 'package:wallet/src/wallet_constants.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

class MockOrganizationDetailBloc extends MockBloc<OrganizationDetailEvent, OrganizationDetailState>
    implements OrganizationDetailBloc {}

void main() {
  const organizationWithoutWrprc = Organization(
    id: 'reader-certificate',
    displayName: 'Example reader',
    legalName: 'Example B.V.',
    personType: PersonType.legalPerson,
    countryCode: 'NL',
    organizationId: 'NTRNL-12345678',
    supportUri: kWrprcSupportUnavailableUri,
  );

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

    testGoldens('OrganizationDetailSuccess light - natural person', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        OrganizationDetailScreen(
          onReportIssuePressed: () {},
        ).withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          OrganizationDetailSuccess(
            organization: WalletMockData.naturalPersonOrganization,
            sharedDataWithOrganizationBefore: false,
          ),
        ),
      );

      await screenMatchesGolden('success.natural_person.light');
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
      expect(find.text(l10n.organizationDetailScreenOrganizationTypeNonGovernment), findsOneWidget);
      expect(find.text(l10n.organizationDetailScreenLegalNameInfo), findsOneWidget);
      expect(find.text(l10n.organizationDetailScreenOrganizationIdInfo), findsOneWidget);
      expect(find.text(WalletMockData.organization.supportUri.replaceAll('https://', '')), findsOneWidget);
      expect(find.text(WalletMockData.organization.privacyPolicyUri!.replaceAll('https://', '')), findsOneWidget);
      expect(find.text(WalletMockData.organization.organizationId), findsOneWidget);
      final location = CountryCodeFormatter.format(WalletMockData.organization.countryCode);
      expect(location, equals('Netherlands'));
      expect(find.text(location!), findsOneWidget);
      expect(find.text(WalletMockData.organization.organizationId), findsOneWidget);
      expect(find.text(l10n.organizationDetailScreenWebsiteInfo), findsNothing);
    });

    testWidgets('a public body is shown as a government organization', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const OrganizationDetailScreen().withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          OrganizationDetailSuccess(
            organization: WalletMockData.organization.copyWith(publicBody: true),
            sharedDataWithOrganizationBefore: false,
          ),
        ),
      );

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.organizationDetailScreenOrganizationTypeGovernment), findsOneWidget);
    });

    testWidgets('a natural person is shown with name and identification number labels', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const OrganizationDetailScreen().withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          OrganizationDetailSuccess(
            organization: WalletMockData.naturalPersonOrganization,
            sharedDataWithOrganizationBefore: false,
          ),
        ),
      );

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.organizationDetailScreenNaturalPersonNameInfo), findsOneWidget);
      expect(find.text(WalletMockData.naturalPersonOrganization.legalName), findsOneWidget);
      expect(find.text(l10n.organizationDetailScreenNaturalPersonIdInfo), findsOneWidget);
      expect(find.text(l10n.organizationDetailScreenLegalNameInfo), findsNothing);
      expect(find.text(l10n.organizationDetailScreenOrganizationIdInfo), findsNothing);
    });

    testWidgets('an email support address is shown without the mailto scheme', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const OrganizationDetailScreen().withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          OrganizationDetailSuccess(
            organization: WalletMockData.organization.copyWith(supportUri: 'mailto:support@example.org'),
            sharedDataWithOrganizationBefore: false,
          ),
        ),
      );

      expect(find.text('support@example.org'), findsOneWidget);
    });

    testWidgets('missing WRPRC details show unavailable content without links', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const OrganizationDetailScreen().withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          const OrganizationDetailSuccess(
            organization: organizationWithoutWrprc,
            sharedDataWithOrganizationBefore: false,
          ),
        ),
        surfaceSize: const Size(375, 1100),
      );
      final l10n = await TestUtils.englishLocalizations;
      final unavailable = find.text(l10n.organizationDetailScreenNotAvailable, findRichText: true);
      expect(unavailable, findsNWidgets(5));
      expect(find.byWidgetPredicate((widget) => widget is Semantics && widget.properties.link == true), findsNothing);
      expect(find.text(kWrprcSupportUnavailableUri), findsNothing);
      expect(find.text(l10n.organizationDetailScreenTitle(organizationWithoutWrprc.displayName)), findsOneWidget);
      expect(find.text(organizationWithoutWrprc.legalName), findsOneWidget);
      expect(find.text(organizationWithoutWrprc.organizationId), findsOneWidget);
      expect(find.text('Netherlands'), findsOneWidget);
    });

    testWidgets('organization type is hidden when it is unknown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const OrganizationDetailScreen().withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          OrganizationDetailSuccess(
            organization: WalletMockData.organization.copyWith(publicBody: null),
            sharedDataWithOrganizationBefore: false,
          ),
        ),
      );

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.organizationDetailScreenCategoryInfo), findsNothing);
    });

    testWidgets('url rows expose their title as label and the displayed url as value of a link', (tester) async {
      final semanticsHandle = tester.ensureSemantics();
      final organization = WalletMockData.organization.copyWith(webUri: 'https://example.org/web');
      await tester.pumpWidgetWithAppWrapper(
        const OrganizationDetailScreen().withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          OrganizationDetailSuccess(organization: organization, sharedDataWithOrganizationBefore: false),
        ),
        surfaceSize: const Size(375, 1200),
      );

      final l10n = await TestUtils.englishLocalizations;
      final rows = {
        l10n.organizationDetailScreenWebsiteInfo: organization.webUri!,
        l10n.organizationDetailScreenSupportInfo: organization.supportUri,
        l10n.organizationDetailScreenPrivacyInfo: organization.privacyPolicyUri!,
      };
      for (final MapEntry(key: title, value: url) in rows.entries) {
        final rowFinder = find.semantics.byLabel(title);
        expect(rowFinder, findsOne);
        final row = rowFinder.evaluate().single;
        expect(row, isSemantics(isLink: true, isButton: false, hasTapAction: true));
        expect(row.getSemanticsData().value, url.replaceAll('https://', ''));
        expect(row.getSemanticsData().linkUrl, Uri.parse(url));
      }
      expect(find.semantics.byLabel(RegExp('https://')), findsNothing);
      semanticsHandle.dispose();
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

    testWidgets('activating a url row opens its url', (tester) async {
      final semanticsHandle = tester.ensureSemantics();
      final launchedUrls = <String>[];
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
        const MethodChannel('plugins.flutter.io/url_launcher'),
        (MethodCall methodCall) async {
          launchedUrls.add(methodCall.arguments['url'] as String);
          return true;
        },
      );
      addTearDown(
        () => TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
          const MethodChannel('plugins.flutter.io/url_launcher'),
          null,
        ),
      );
      final organization = WalletMockData.organization.copyWith(webUri: 'https://example.org/web');
      await tester.pumpWidgetWithAppWrapper(
        const OrganizationDetailScreen().withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          OrganizationDetailSuccess(organization: organization, sharedDataWithOrganizationBefore: false),
        ),
        surfaceSize: const Size(375, 1200),
      );

      final l10n = await TestUtils.englishLocalizations;
      tester.semantics.tap(find.semantics.byLabel(l10n.organizationDetailScreenWebsiteInfo));
      await tester.pumpAndSettle();

      expect(launchedUrls, [organization.webUri]);
      semanticsHandle.dispose();
    });
  });
}
