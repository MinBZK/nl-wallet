import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/attribute/requested_attribute.dart';
import 'package:wallet/src/domain/usecase/pin/confirm_transaction_usecase.dart';
import 'package:wallet/src/feature/disclosure/bloc/disclosure_bloc.dart';
import 'package:wallet/src/feature/disclosure/disclosure_screen.dart';
import 'package:wallet/src/feature/disclosure/model/disclosure_flow.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/mock_data.dart';
import '../../util/device_utils.dart';
import '../../util/test_utils.dart';
import '../pin/pin_page_test.dart';

class MockDisclosureBloc extends MockBloc<DisclosureEvent, DisclosureState> implements DisclosureBloc {}

class MockConfirmTransactionUseCase implements ConfirmTransactionUseCase {
  @override
  Future<CheckPinResult> invoke(String pin) => throw UnimplementedError();
}

void main() {
  final DisclosureFlow mockFlow = DisclosureFlow(
    id: 'id',
    organization: WalletMockData.organization,
    hasPreviouslyInteractedWithOrganization: false,
    availableAttributes: {
      WalletMockData.card: const [WalletMockData.textDataAttribute]
    },
    requestedAttributes: [
      RequestedAttribute(
        label: 'name',
        key: 'WalletMockData.textDataAttribute.type',
        valueType: WalletMockData.textDataAttribute.valueType,
      )
    ],
    requestPurpose: 'Purpose goes here',
    policy: WalletMockData.policy,
  );

  group('goldens', () {
    testGoldens('DisclosureInitial Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
              MockDisclosureBloc(),
              DisclosureInitial(),
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
              DisclosureCheckOrganization(mockFlow),
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
              DisclosureGenericError(),
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
            widget: RepositoryProvider<ConfirmTransactionUseCase>.value(
              value: MockConfirmTransactionUseCase(),
              child: const DisclosureScreen()
                  .withState<DisclosureBloc, DisclosureState>(
                    MockDisclosureBloc(),
                    DisclosureConfirmPin(mockFlow),
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
              DisclosureMissingAttributes(mockFlow),
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
              DisclosureConfirmDataAttributes(mockFlow),
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
              DisclosureSuccess(mockFlow),
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
              const DisclosureStopped(),
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
          DisclosureCheckOrganization(mockFlow),
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
    testWidgets('show history button is shown on the success page', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
          MockDisclosureBloc(),
          DisclosureSuccess(mockFlow),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.verificationScreenShowHistoryCta), findsOneWidget);
    });
  });
}
