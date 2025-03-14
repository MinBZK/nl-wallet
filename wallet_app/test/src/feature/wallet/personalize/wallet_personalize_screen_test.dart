import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/flow_progress.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/pid/accept_offered_pid_usecase.dart';
import 'package:wallet/src/feature/common/page/generic_loading_page.dart';
import 'package:wallet/src/feature/common/page/terminal_page.dart';
import 'package:wallet/src/feature/error/error_page.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/wallet/personalize/bloc/wallet_personalize_bloc.dart';
import 'package:wallet/src/feature/wallet/personalize/page/wallet_personalize_check_data_offering_page.dart';
import 'package:wallet/src/feature/wallet/personalize/page/wallet_personalize_confirm_pin_page.dart';
import 'package:wallet/src/feature/wallet/personalize/page/wallet_personalize_intro_page.dart';
import 'package:wallet/src/feature/wallet/personalize/page/wallet_personalize_success_page.dart';
import 'package:wallet/src/feature/wallet/personalize/wallet_personalize_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../mocks/wallet_mocks.dart';
import '../../../util/device_utils.dart';
import '../../../util/test_utils.dart';
import '../../pin/pin_page_test.dart';

class MockWalletPersonalizeBloc extends MockBloc<WalletPersonalizeEvent, WalletPersonalizeState>
    implements WalletPersonalizeBloc {}

void main() {
  const kPidId = 'id';

  /// All attributes here are needed to satisfy the [PidAttributeMapper] used when rendering the [WalletPersonalizeCheckData] state.
  final sampleMaleAttributes = [
    DataAttribute.untranslated(
      label: 'Voornamen',
      value: const StringValue('John'),
      key: 'mock.firstNames',
      sourceCardDocType: kPidId,
    ),
    DataAttribute.untranslated(
      label: 'Achternaam',
      value: const StringValue('Doe'),
      key: 'mock.lastName',
      sourceCardDocType: kPidId,
    ),
    DataAttribute.untranslated(
      label: 'Naam bij geboorte',
      value: const StringValue('John'),
      key: 'mock.birthName',
      sourceCardDocType: kPidId,
    ),
    DataAttribute.untranslated(
      label: 'Geboortedatum',
      value: const StringValue('01-01-2023'),
      key: 'mock.birthDate',
      sourceCardDocType: kPidId,
    ),
    DataAttribute.untranslated(
      label: 'Geboorteplaats',
      value: const StringValue('Amsterdam'),
      key: 'mock.birthPlace',
      sourceCardDocType: kPidId,
    ),
    DataAttribute.untranslated(
      label: 'Geboorteland',
      value: const StringValue('Nederland'),
      key: 'mock.birthCountry',
      sourceCardDocType: kPidId,
    ),
    DataAttribute.untranslated(
      label: 'Getrouwd of geregistreerd partnerschap',
      value: const StringValue('Nee'),
      key: 'mock.hasSpouseOrPartner',
      sourceCardDocType: kPidId,
    ),
    DataAttribute.untranslated(
      label: 'Burger­service­nummer (BSN)',
      value: const StringValue('111222333'),
      key: 'mock.citizenshipNumber',
      sourceCardDocType: kPidId,
    ),
    DataAttribute.untranslated(
      label: 'Stad',
      value: const StringValue('Amsterdam'),
      key: 'mock.city',
      sourceCardDocType: kPidId,
    ),
    DataAttribute.untranslated(
      label: 'Postcode',
      value: const StringValue('1234AB'),
      key: 'mock.postalCode',
      sourceCardDocType: kPidId,
    ),
    DataAttribute.untranslated(
      label: 'Straatnaam',
      value: const StringValue('Dorpsstraat'),
      key: 'mock.streetName',
      sourceCardDocType: kPidId,
    ),
    DataAttribute.untranslated(
      label: 'Huisnummer',
      value: const StringValue('1A'),
      key: 'mock.houseNumber',
      sourceCardDocType: kPidId,
    ),
  ];

  group('goldens', () {
    testGoldens('WalletPersonalizeInitial Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
              MockWalletPersonalizeBloc(),
              const WalletPersonalizeInitial(),
            ),
            name: 'initial',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/initial.light');
    });

    testGoldens('WalletPersonalizeLoadingIssuanceUrl Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
              MockWalletPersonalizeBloc(),
              const WalletPersonalizeLoadingIssuanceUrl(),
            ),
            name: 'loading_issuance_url',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/loading_issuance_url.light');
    });

    testGoldens('WalletPersonalizeLoadInProgress Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
              MockWalletPersonalizeBloc(),
              const WalletPersonalizeLoadInProgress(FlowProgress(currentStep: 1, totalSteps: 2)),
            ),
            name: 'load_in_progress',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/load_in_progress.light');
    });

    testGoldens('WalletPersonalizeAddingCards Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          const WalletPersonalizeAddingCards(FlowProgress(currentStep: 8, totalSteps: 9)),
        ),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/adding_cards.light');
    });

    testGoldens('WalletPersonalizeAuthenticating Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
              MockWalletPersonalizeBloc(),
              const WalletPersonalizeAuthenticating(),
            ),
            name: 'authenticating',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/authenticating.light');
    });

    testGoldens('WalletPersonalizeConnectDigid Light', (tester) async {
      const mockUrl = 'https://digid_login';
      bool mockUrlIsOpened = false;

      // Mock the launchUrl plugin and check if the mockUrl comes in
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
        const MethodChannel('plugins.flutter.io/url_launcher'),
        (MethodCall methodCall) async {
          mockUrlIsOpened = methodCall.arguments['url'] == mockUrl;
          return null;
        },
      );

      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          const WalletPersonalizeConnectDigid(mockUrl),
        ),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/connect_digid.light');

      // Verify that the mockUrl was passed to the url_launcher plugin
      expect(mockUrlIsOpened, isTrue);
    });

    testGoldens(
      'WalletPersonalizeAuthenticating - Cancel Dialog - Light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
            MockWalletPersonalizeBloc(),
            const WalletPersonalizeAuthenticating(),
          ),
        );
        final l10n = await TestUtils.englishLocalizations;
        final cancelButtonFinder = find.text(l10n.walletPersonalizeScreenDigidLoadingStopCta);
        await tester.tap(cancelButtonFinder);
        await tester.pumpAndSettle();
        await screenMatchesGolden(tester, 'wallet_personalize/authenticating.cancel_dialog.light');
      },
    );

    testGoldens('WalletPersonalizeConfirmPin Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: RepositoryProvider<AcceptOfferedPidUseCase>.value(
              value: Mocks.create<AcceptOfferedPidUseCase>(),
              child: const WalletPersonalizeScreen()
                  .withState<WalletPersonalizeBloc, WalletPersonalizeState>(
                    MockWalletPersonalizeBloc(),
                    const WalletPersonalizeConfirmPin([]),
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
      await screenMatchesGolden(tester, 'wallet_personalize/confirm_pin.light');
    });

    testGoldens('WalletPersonalizeCheckData Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
              MockWalletPersonalizeBloc(),
              WalletPersonalizeCheckData(availableAttributes: sampleMaleAttributes),
            ),
            name: 'check_data',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/check_data.light');
    });

    testGoldens('WalletPersonalizeSuccess Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
              MockWalletPersonalizeBloc(),
              WalletPersonalizeSuccess([WalletMockData.card, WalletMockData.altCard]),
            ),
            name: 'success',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/success.light');
    });

    testGoldens('WalletPersonalizeSuccess Dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
              MockWalletPersonalizeBloc(),
              WalletPersonalizeSuccess([WalletMockData.card, WalletMockData.altCard]),
            ),
            name: 'success',
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/success.dark');
    });

    testGoldens('WalletPersonalizeFailure Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
              MockWalletPersonalizeBloc(),
              WalletPersonalizeFailure(),
            ),
            name: 'failure',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/failure.light');
    });

    testGoldens('WalletPersonalizeDigidFailure Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
              MockWalletPersonalizeBloc(),
              const WalletPersonalizeDigidFailure(error: GenericError('', sourceError: 'test')),
            ),
            name: 'digid_failure',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/digid_failure.light');
    });

    testGoldens('WalletPersonalizeDigidCancelled Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
              MockWalletPersonalizeBloc(),
              WalletPersonalizeDigidCancelled(),
            ),
            name: 'digid_cancelled',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/digid_cancelled.light');
    });

    testGoldens('WalletPersonalizeDigidFailure Light Portrait', (tester) async {
      /// This test verifies that the image scaling is correct when rendered in portrait mode, as the
      /// test above (WalletPersonalizeDigidFailure Light) is treated as landscape.
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          const WalletPersonalizeDigidFailure(error: GenericError('', sourceError: 'test')),
        ),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/digid_failure.portrait.light');
    });
  });

  group('widgets', () {
    testWidgets('continue button is shown on the success page', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          WalletPersonalizeSuccess([WalletMockData.card]),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.walletPersonalizeSuccessPageContinueCta), findsOneWidget);
    });

    testWidgets(
      'cancel dialog is dismissed if digid result comes in while it is shown',
      (tester) async {
        // Configure the bloc with a state where the cancel button is visible
        final mockBloc = MockWalletPersonalizeBloc();
        final mockStateStream = BehaviorSubject<WalletPersonalizeState>.seeded(const WalletPersonalizeAuthenticating());
        whenListen(mockBloc, mockStateStream, initialState: mockStateStream.value);

        // Show the loading state (which contains the cancel button)
        await tester.pumpWidgetWithAppWrapper(
          BlocProvider<WalletPersonalizeBloc>(
            create: (c) => mockBloc,
            child: Builder(builder: (context) => const WalletPersonalizeScreen()),
          ),
        );

        // Find the cancel button and tap it
        final l10n = await TestUtils.englishLocalizations;
        final cancelButtonFinder = find.text(l10n.walletPersonalizeScreenDigidLoadingStopCta);
        await tester.tap(cancelButtonFinder);
        await tester.pumpAndSettle();

        // Verify the cancel dialog is shown
        final stopDialogTitleFinder = find.text(l10n.walletPersonalizeScreenStopDigidDialogTitle);
        expect(stopDialogTitleFinder, findsOneWidget);

        // Mock digid result coming in
        mockStateStream.add(WalletPersonalizeCheckData(availableAttributes: sampleMaleAttributes));
        await tester.pumpAndSettle();

        // Verify dialog is gone and confirm attributes screen is shown
        expect(stopDialogTitleFinder, findsNothing);
        // Look for 2 widgets due to usage of [SliverWalletAppBar]
        expect(find.text(l10n.walletPersonalizeCheckDataOfferingPageTitle), findsNWidgets(2));
      },
    );

    testWidgets(
        'WalletPersonalizeScreen shows the no internet error for WalletPersonalizeNetworkError(hasInternet=false)',
        (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          const WalletPersonalizeNetworkError(
            error: NetworkError(hasInternet: false, sourceError: 'test'),
            hasInternet: false,
          ),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the 'no internet' title is shown
      final noInternetHeadlineFinder = find.text(l10n.errorScreenNoInternetHeadline);
      expect(noInternetHeadlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'try again' cta is shown
      final tryAgainCtaFinder = find.text(l10n.generalRetry);
      expect(tryAgainCtaFinder, findsOneWidget);

      // Verify the 'show details' cta is shown
      final showDetailsCtaFinder = find.text(l10n.generalShowDetailsCta);
      expect(showDetailsCtaFinder, findsOneWidget);
    });

    testWidgets(
        'WalletPersonalizeScreen shows the no internet error for WalletPersonalizeNetworkError(hasInternet=true)',
        (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          const WalletPersonalizeNetworkError(
            error: NetworkError(hasInternet: true, sourceError: 'test'),
            hasInternet: true,
          ),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the 'server error' title is shown
      final noInternetHeadlineFinder = find.text(l10n.errorScreenServerHeadline);
      expect(noInternetHeadlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'try again' cta is shown
      final tryAgainCtaFinder = find.text(l10n.generalRetry);
      expect(tryAgainCtaFinder, findsOneWidget);

      // Verify the 'show details' cta is shown
      final showDetailsCtaFinder = find.text(l10n.generalShowDetailsCta);
      expect(showDetailsCtaFinder, findsOneWidget);
    });

    testWidgets('WalletPersonalizeScreen shows the generic error for SetupSecurityGenericError state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          const WalletPersonalizeGenericError(
            error: GenericError('generic', sourceError: 'test'),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the 'something went wrong' title is shown
      final headlineFinder = find.text(l10n.errorScreenGenericHeadline);
      expect(headlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'try again' cta is shown
      final tryAgainCtaFinder = find.text(l10n.generalRetry);
      expect(tryAgainCtaFinder, findsOneWidget);

      // Verify the 'show details' cta is shown
      final showDetailsCtaFinder = find.text(l10n.generalShowDetailsCta);
      expect(showDetailsCtaFinder, findsOneWidget);
    });

    testWidgets('WalletPersonalizeScreen shows session expired for WalletPersonalizeSessionExpired state',
        (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          const WalletPersonalizeSessionExpired(
            error: SessionError(state: SessionState.expired, sourceError: 'test'),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the 'session expired' title is shown
      final headlineFinder = find.text(l10n.errorScreenSessionExpiredHeadline);
      expect(headlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'try again' cta is shown
      final tryAgainCtaFinder = find.text(l10n.generalRetry);
      expect(tryAgainCtaFinder, findsOneWidget);

      // Verify the 'show details' cta is shown
      final showDetailsCtaFinder = find.text(l10n.generalShowDetailsCta);
      expect(showDetailsCtaFinder, findsOneWidget);
    });

    testWidgets('Verify WalletPersonalizeInitial shows WalletPersonalizeIntroPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          const WalletPersonalizeInitial(),
        ),
      );
      expect(find.byType(WalletPersonalizeIntroPage), findsOneWidget);
    });

    testWidgets('Verify WalletPersonalizeLoadingIssuanceUrl shows GenericLoadingPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          const WalletPersonalizeLoadingIssuanceUrl(),
        ),
      );
      expect(find.byType(GenericLoadingPage), findsOneWidget);
    });

    testWidgets('Verify WalletPersonalizeConnectDigid shows GenericLoadingPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          const WalletPersonalizeConnectDigid('auth_url'),
        ),
      );
      expect(find.byType(GenericLoadingPage), findsOneWidget);
    });

    testWidgets('Verify WalletPersonalizeAuthenticating shows GenericLoadingPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          const WalletPersonalizeAuthenticating(),
        ),
      );
      expect(find.byType(GenericLoadingPage), findsOneWidget);
    });

    testWidgets('Verify WalletPersonalizeLoadInProgress shows GenericLoadingPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          const WalletPersonalizeLoadInProgress(FlowProgress(totalSteps: 5, currentStep: 1)),
        ),
      );
      expect(find.byType(GenericLoadingPage), findsOneWidget);
    });

    testWidgets('Verify WalletPersonalizeCheckData shows y', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          WalletPersonalizeCheckData(availableAttributes: sampleMaleAttributes),
        ),
      );
      expect(find.byType(WalletPersonalizeCheckDataOfferingPage), findsOneWidget);
    });

    testWidgets('Verify WalletPersonalizeConfirmPin shows WalletPersonalizeConfirmPinPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          WalletPersonalizeConfirmPin(sampleMaleAttributes),
        ),
        providers: [
          RepositoryProvider<PinBloc>(create: (_) => MockPinBloc()),
          RepositoryProvider<AcceptOfferedPidUseCase>(create: (_) => MockAcceptOfferedPidUseCase()),
        ],
      );
      expect(find.byType(WalletPersonalizeConfirmPinPage), findsOneWidget);
    });

    testWidgets('Verify WalletPersonalizeSuccess shows y', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          WalletPersonalizeSuccess([WalletMockData.card, WalletMockData.altCard]),
        ),
      );
      expect(find.byType(WalletPersonalizeSuccessPage), findsOneWidget);
    });

    testWidgets('Verify WalletPersonalizeFailure shows TerminalPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          WalletPersonalizeFailure(),
        ),
      );
      expect(find.byType(TerminalPage), findsOneWidget);
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.walletPersonalizeScreenErrorTitle), findsOneWidget);
    });

    testWidgets('Verify WalletPersonalizeDigidCancelled shows WalletPersonalizeDigidErrorPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          WalletPersonalizeDigidCancelled(),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.byType(TerminalPage), findsOneWidget);
      expect(find.text(l10n.walletPersonalizeDigidCancelledPageTitle, findRichText: true), findsOneWidget);
    });

    testWidgets('Verify WalletPersonalizeDigidFailure shows WalletPersonalizeDigidErrorPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          const WalletPersonalizeDigidFailure(error: GenericError('', sourceError: 'test')),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.byType(TerminalPage), findsOneWidget);
      expect(find.text(l10n.walletPersonalizeDigidErrorPageTitle, findRichText: true), findsOneWidget);
    });

    testWidgets('Verify WalletPersonalizeNetworkError shows ErrorPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          const WalletPersonalizeNetworkError(
            error: NetworkError(hasInternet: true, sourceError: 'test'),
            hasInternet: true,
          ),
        ),
      );
      expect(find.byType(ErrorPage), findsOneWidget);
    });

    testWidgets('Verify WalletPersonalizeGenericError shows ErrorPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          const WalletPersonalizeGenericError(error: GenericError('generic', sourceError: 'test')),
        ),
      );
      expect(find.byType(ErrorPage), findsOneWidget);
    });

    testWidgets('Verify WalletPersonalizeSessionExpired shows ErrorPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          const WalletPersonalizeSessionExpired(
            error: SessionError(state: SessionState.expired, sourceError: 'test'),
          ),
        ),
      );
      expect(find.byType(ErrorPage), findsOneWidget);
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.errorScreenSessionExpiredHeadline), findsOneWidget);
    });

    testWidgets('Verify WalletPersonalizeAddingCards shows dedicated loading message', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
          MockWalletPersonalizeBloc(),
          const WalletPersonalizeAddingCards(FlowProgress(currentStep: 8, totalSteps: 9)),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.walletPersonalizeScreenAddingCardsSubtitle), findsOneWidget);
    });
  });
}
