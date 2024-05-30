import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/attribute/missing_attribute.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_session_type.dart';
import 'package:wallet/src/domain/usecase/disclosure/accept_disclosure_usecase.dart';
import 'package:wallet/src/feature/disclosure/bloc/disclosure_bloc.dart';
import 'package:wallet/src/feature/disclosure/disclosure_screen.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mock_data.dart';
import '../../mocks/wallet_mocks.dart';
import '../../util/device_utils.dart';
import '../../util/test_utils.dart';
import '../pin/pin_page_test.dart';

class MockDisclosureBloc extends MockBloc<DisclosureEvent, DisclosureState> implements DisclosureBloc {}

void main() {
  group('goldens', () {
    testGoldens('DisclosureInitial Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
              MockDisclosureBloc(),
              const DisclosureInitial(),
            ),
            name: 'initial',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'initial.light');
    });

    testGoldens('DisclosureLoadInProgress Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
              MockDisclosureBloc(),
              DisclosureLoadInProgress(),
            ),
            name: 'load_in_progress',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'load_in_progress.light');
    });

    testGoldens('DisclosureCheckOrganization Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
              MockDisclosureBloc(),
              DisclosureCheckOrganization(
                relyingParty: WalletMockData.organization,
                originUrl: 'http://origin.org',
                sharedDataWithOrganizationBefore: true,
                sessionType: DisclosureSessionType.crossDevice,
              ),
            ),
            name: 'check_organization',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'check_organization.light');
    });

    testGoldens('DisclosureGenericError Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
              MockDisclosureBloc(),
              const DisclosureGenericError(error: anything),
            ),
            name: 'generic_error',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'generic_error.light');
    });

    testGoldens('DisclosureConfirmPin Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: RepositoryProvider<AcceptDisclosureUseCase>.value(
              value: MockAcceptDisclosureUseCase(),
              child: const DisclosureScreen()
                  .withState<DisclosureBloc, DisclosureState>(
                    MockDisclosureBloc(),
                    DisclosureConfirmPin(relyingParty: WalletMockData.organization),
                  )
                  .withState<PinBloc, PinState>(
                    MockPinBloc(),
                    const PinEntryInProgress(0),
                  ),
            ),
            name: 'confirm_pin',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'confirm_pin.light');
    });

    testGoldens('DisclosureMissingAttributes Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
              MockDisclosureBloc(),
              DisclosureMissingAttributes(
                relyingParty: WalletMockData.organization,
                missingAttributes: [MissingAttribute(label: 'missing'.untranslated)],
              ),
            ),
            name: 'missing_attributes',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'missing_attributes.light');
    });

    testGoldens('DisclosureConfirmDataAttributes Dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
              MockDisclosureBloc(),
              DisclosureConfirmDataAttributes(
                relyingParty: WalletMockData.organization,
                requestedAttributes: {
                  WalletMockData.card: [WalletMockData.textDataAttribute]
                },
                requestPurpose: 'Sample reason'.untranslated,
                policy: WalletMockData.policy,
              ),
            ),
            name: 'confirm_data_attributes',
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'confirm_data_attributes.dark');
    });

    testGoldens('DisclosureSuccess Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
              MockDisclosureBloc(),
              DisclosureSuccess(relyingParty: WalletMockData.organization),
            ),
            name: 'success',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'success.light');
    });

    testGoldens('DisclosureStopped Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
              MockDisclosureBloc(),
              DisclosureStopped(organization: WalletMockData.organization),
            ),
            name: 'stopped',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'stopped.light');
    });

    testGoldens('DisclosureLeftFeedback Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
              MockDisclosureBloc(),
              const DisclosureLeftFeedback(),
            ),
            name: 'left_feedback',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'left_feedback.light');
    });

    testGoldens('Disclosure Stop Sheet Light', (tester) async {
      // Inflate a state with showStopConfirmation = true.
      await tester.pumpWidgetWithAppWrapper(
        const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
          MockDisclosureBloc(),
          DisclosureCheckOrganization(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            sharedDataWithOrganizationBefore: true,
            sessionType: DisclosureSessionType.crossDevice,
          ),
        ),
      );
      // Find and press the close button
      final closeButtonFinder = find.byIcon(Icons.close);
      await tester.tap(closeButtonFinder);
      await tester.pumpAndSettle();

      await screenMatchesGolden(tester, 'stop_sheet.light');
    });
  });

  group('widgets', () {
    testWidgets('when cross-device session; fraud warning is shown on organization approve page', (tester) async {
      const originUrl = 'http://origin.org';

      await tester.pumpWidgetWithAppWrapper(
        const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
          MockDisclosureBloc(),
          DisclosureCheckOrganization(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            sharedDataWithOrganizationBefore: true,
            sessionType: DisclosureSessionType.crossDevice,
          ),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.organizationApprovePageFraudInfoPart1, findRichText: true), findsOneWidget);
      expect(find.text(l10n.organizationApprovePageFraudInfoPart2(originUrl), findRichText: true), findsOneWidget);
    });

    testWidgets('when same-device session; fraud warning is NOT shown on organization approve page', (tester) async {
      const originUrl = 'http://origin.org';

      await tester.pumpWidgetWithAppWrapper(
        const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
          MockDisclosureBloc(),
          DisclosureCheckOrganization(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            sharedDataWithOrganizationBefore: true,
            sessionType: DisclosureSessionType.sameDevice,
          ),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.organizationApprovePageFraudInfoPart1, findRichText: true), findsNothing);
      expect(find.text(l10n.organizationApprovePageFraudInfoPart2(originUrl), findRichText: true), findsNothing);
    });

    testWidgets('history button is shown on the success page', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
          MockDisclosureBloc(),
          DisclosureSuccess(relyingParty: WalletMockData.organization),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.disclosureSuccessPageShowHistoryCta), findsOneWidget);
    });

    testWidgets('DisclosureScreen shows the no internet error for DisclosureNetworkError(hasInternet=false)',
        (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
          MockDisclosureBloc(),
          const DisclosureNetworkError(error: CoreNetworkError('no internet'), hasInternet: false),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;

      await tester.pumpAndSettle();

      // Verify the 'no internet' title is shown
      final noInternetHeadlineFinder = find.text(l10n.errorScreenNoInternetHeadline);
      expect(noInternetHeadlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'close' cta is shown
      final closeCtaFinder = find.text(l10n.generalClose);
      expect(closeCtaFinder, findsOneWidget);

      // Verify the 'close' icon is shown
      final closeIconFinder = find.byIcon(Icons.close_outlined);
      expect(closeIconFinder, findsOneWidget);

      // Verify the 'show details' cta is shown
      final showDetailsCtaFinder = find.text(l10n.generalShowDetailsCta);
      expect(showDetailsCtaFinder, findsOneWidget);
    });

    testWidgets('DisclosureScreen shows the server error for DisclosureNetworkError(hasInternet=true)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
          MockDisclosureBloc(),
          const DisclosureNetworkError(error: CoreNetworkError('server'), hasInternet: true),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;

      await tester.pumpAndSettle();

      // Verify the 'no internet' title is shown
      final noInternetHeadlineFinder = find.text(l10n.errorScreenServerHeadline);
      expect(noInternetHeadlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'close' cta is shown
      final closeCtaFinder = find.text(l10n.generalClose);
      expect(closeCtaFinder, findsOneWidget);

      // Verify the 'close' icon is shown
      final closeIconFinder = find.byIcon(Icons.close_outlined);
      expect(closeIconFinder, findsOneWidget);

      // Verify the 'show details' cta is shown
      final showDetailsCtaFinder = find.text(l10n.generalShowDetailsCta);
      expect(showDetailsCtaFinder, findsOneWidget);
    });

    testWidgets('DisclosureScreen shows the generic error for CoreGenericError', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
          MockDisclosureBloc(),
          const DisclosureGenericError(error: CoreGenericError('generic')),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the 'something went wrong' title is shown
      final headlineFinder = find.text(l10n.errorScreenGenericHeadline);
      expect(headlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'close' cta is shown
      final closeCtaFinder = find.text(l10n.generalClose);
      expect(closeCtaFinder, findsOneWidget);

      // Verify the 'close' icon is shown
      final closeIconFinder = find.byIcon(Icons.close_outlined);
      expect(closeIconFinder, findsOneWidget);

      // Verify the 'show details' cta is shown
      final showDetailsCtaFinder = find.text(l10n.generalShowDetailsCta);
      expect(showDetailsCtaFinder, findsOneWidget);
    });

    testWidgets('DisclosureScreen shows the generic error for CoreGenericError', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
          MockDisclosureBloc(),
          const DisclosureGenericError(error: CoreGenericError('generic')),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the 'something went wrong' title is shown
      final headlineFinder = find.text(l10n.errorScreenGenericHeadline);
      expect(headlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'close' cta is shown
      final closeCtaFinder = find.text(l10n.generalClose);
      expect(closeCtaFinder, findsOneWidget);

      // Verify the 'close' icon is shown
      final closeIconFinder = find.byIcon(Icons.close_outlined);
      expect(closeIconFinder, findsOneWidget);
    });
  });
}
