import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/domain/model/attribute/data_attribute.dart';
import 'package:wallet/src/domain/usecase/pid/accept_offered_pid_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/confirm_transaction_usecase.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/wallet/personalize/bloc/wallet_personalize_bloc.dart';
import 'package:wallet/src/feature/wallet/personalize/wallet_personalize_screen.dart';
import 'package:wallet/src/util/mapper/pid/pid_attributes_mapper.dart';
import 'package:wallet/src/util/mapper/pid/pid_data_attributes_mapper.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/mock_data.dart';
import '../../../mocks/wallet_mocks.dart';
import '../../../util/device_utils.dart';
import '../../../util/test_utils.dart';
import '../../pin/pin_page_test.dart';

class MockWalletPersonalizeBloc extends MockBloc<WalletPersonalizeEvent, WalletPersonalizeState>
    implements WalletPersonalizeBloc {}

class MockConfirmTransactionUseCase implements ConfirmTransactionUseCase {
  @override
  Future<CheckPinResult> invoke(String pin) => throw UnimplementedError();
}

void main() {
  const kPidId = 'id';

  /// All attributes here are needed to satisfy the [PidAttributeMapper] used when rendering the [WalletPersonalizeCheckData] state.
  const pidAttributes = [
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Voornamen',
      value: 'John',
      key: 'mock.firstNames',
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Achternaam',
      value: 'Doe',
      key: 'mock.lastName',
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Geboortenaam',
      value: 'John',
      key: 'mock.birthName',
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Geslacht',
      value: 'Male',
      key: 'mock.gender',
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Geboortedatum',
      value: '01-01-2000',
      key: 'mock.birthDate',
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Geboorteplaats',
      value: 'Amsterdam',
      key: 'mock.birthPlace',
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Geboorteland',
      value: 'Nederland',
      key: 'mock.birthCountry',
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Burgerservicenummer (BSN)',
      value: '******999',
      key: 'mock.citizenshipNumber',
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Nationaliteit',
      value: 'Nederlands',
      key: 'mock.nationality',
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Stad',
      value: 'Amsterdam',
      key: 'mock.city',
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Postcode',
      value: '1234AB',
      key: 'mock.postalCode',
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Straatnaam',
      value: 'Dorpsstraat',
      key: 'mock.streetName',
      sourceCardId: kPidId,
    ),
    DataAttribute(
      valueType: AttributeValueType.text,
      label: 'Huisnummer',
      value: '1A',
      key: 'mock.houseNumber',
      sourceCardId: kPidId,
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
              const WalletPersonalizeLoadInProgress(5),
            ),
            name: 'load_in_progress',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize/load_in_progress.light');
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
        ));
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
            widget: RepositoryProvider<PidAttributeMapper>(
              create: (c) => PidDataAttributeMapper(),
              child: const WalletPersonalizeScreen().withState<WalletPersonalizeBloc, WalletPersonalizeState>(
                MockWalletPersonalizeBloc(),
                const WalletPersonalizeCheckData(availableAttributes: pidAttributes),
              ),
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
              const WalletPersonalizeSuccess([WalletMockData.card, WalletMockData.altCard]),
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
              const WalletPersonalizeSuccess([WalletMockData.card, WalletMockData.altCard]),
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
              WalletPersonalizeDigidFailure(),
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
          WalletPersonalizeDigidFailure(),
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
          const WalletPersonalizeSuccess([WalletMockData.card]),
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
          RepositoryProvider<PidAttributeMapper>(
            create: (c) => PidDataAttributeMapper(),
            child: BlocProvider<WalletPersonalizeBloc>(
              create: (c) => mockBloc,
              child: Builder(builder: (context) => const WalletPersonalizeScreen()),
            ),
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
        mockStateStream.add(const WalletPersonalizeCheckData(availableAttributes: pidAttributes));
        await tester.pumpAndSettle();

        // Verify dialog is gone and confirm attributes screen is shown
        expect(stopDialogTitleFinder, findsNothing);
        expect(find.text(l10n.walletPersonalizeCheckDataOfferingPageTitle), findsOneWidget);
      },
    );
  });
}
