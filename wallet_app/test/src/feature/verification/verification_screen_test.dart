import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/attribute/requested_attribute.dart';
import 'package:wallet/src/domain/usecase/pin/confirm_transaction_usecase.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/verification/bloc/verification_bloc.dart';
import 'package:wallet/src/feature/verification/model/verification_flow.dart';
import 'package:wallet/src/feature/verification/verification_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/mock_data.dart';
import '../../util/device_utils.dart';
import '../../util/test_utils.dart';
import '../pin/pin_page_test.dart';

class MockVerificationBloc extends MockBloc<VerificationEvent, VerificationState> implements VerificationBloc {}

class MockConfirmTransactionUseCase implements ConfirmTransactionUseCase {
  @override
  Future<CheckPinResult> invoke(String pin) => throw UnimplementedError();
}

void main() {
  final VerificationFlow mockFlow = VerificationFlow(
    id: 'id',
    organization: WalletMockData.organization,
    hasPreviouslyInteractedWithOrganization: false,
    availableAttributes: {
      WalletMockData.card: const [WalletMockData.textDataAttribute]
    },
    requestedAttributes: [
      RequestedAttribute(
        name: 'name',
        type: WalletMockData.textDataAttribute.type,
        valueType: WalletMockData.textDataAttribute.valueType,
      )
    ],
    requestPurpose: 'Purpose goes here',
    policy: WalletMockData.policy,
  );

  group('goldens', () {
    testGoldens('VerificationInitial Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const VerificationScreen().withState<VerificationBloc, VerificationState>(
              MockVerificationBloc(),
              VerificationInitial(),
            ),
            name: 'initial',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'initial.light');
    });

    testGoldens('VerificationLoadInProgress Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const VerificationScreen().withState<VerificationBloc, VerificationState>(
              MockVerificationBloc(),
              VerificationLoadInProgress(),
            ),
            name: 'load_in_progress',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'load_in_progress.light');
    });

    testGoldens('VerificationCheckOrganization Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const VerificationScreen().withState<VerificationBloc, VerificationState>(
              MockVerificationBloc(),
              VerificationCheckOrganization(mockFlow),
            ),
            name: 'check_organization',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'check_organization.light');
    });

    testGoldens('VerificationGenericError Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const VerificationScreen().withState<VerificationBloc, VerificationState>(
              MockVerificationBloc(),
              VerificationGenericError(),
            ),
            name: 'generic_error',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'generic_error.light');
    });

    testGoldens('VerificationConfirmPin Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: RepositoryProvider<ConfirmTransactionUseCase>.value(
              value: MockConfirmTransactionUseCase(),
              child: const VerificationScreen()
                  .withState<VerificationBloc, VerificationState>(
                    MockVerificationBloc(),
                    VerificationConfirmPin(mockFlow),
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

    testGoldens('VerificationMissingAttributes Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const VerificationScreen().withState<VerificationBloc, VerificationState>(
              MockVerificationBloc(),
              VerificationMissingAttributes(mockFlow),
            ),
            name: 'missing_attributes',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'missing_attributes.light');
    });

    testGoldens('VerificationConfirmDataAttributes Dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const VerificationScreen().withState<VerificationBloc, VerificationState>(
              MockVerificationBloc(),
              VerificationConfirmDataAttributes(mockFlow),
            ),
            name: 'confirm_data_attributes',
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'confirm_data_attributes.dark');
    });

    testGoldens('VerificationSuccess Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const VerificationScreen().withState<VerificationBloc, VerificationState>(
              MockVerificationBloc(),
              VerificationSuccess(mockFlow),
            ),
            name: 'success',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'success.light');
    });

    testGoldens('VerificationStopped Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const VerificationScreen().withState<VerificationBloc, VerificationState>(
              MockVerificationBloc(),
              const VerificationStopped(),
            ),
            name: 'verification_stopped',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'verification_stopped.light');
    });

    testGoldens('VerificationLeftFeedback Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const VerificationScreen().withState<VerificationBloc, VerificationState>(
              MockVerificationBloc(),
              const VerificationLeftFeedback(),
            ),
            name: 'left_feedback',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'left_feedback.light');
    });

    testGoldens('Verification Stop Sheet Light', (tester) async {
      // Inflate a state with showStopConfirmation = true.
      await tester.pumpWidgetWithAppWrapper(
        const VerificationScreen().withState<VerificationBloc, VerificationState>(
          MockVerificationBloc(),
          VerificationCheckOrganization(mockFlow),
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
        const VerificationScreen().withState<VerificationBloc, VerificationState>(
          MockVerificationBloc(),
          VerificationSuccess(mockFlow),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.verificationScreenShowHistoryCta), findsOneWidget);
    });
  });
}
