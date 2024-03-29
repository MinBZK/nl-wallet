import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/pin/pin_validation_error.dart';
import 'package:wallet/src/feature/setup_security/bloc/setup_security_bloc.dart';
import 'package:wallet/src/feature/setup_security/setup_security_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';

class MockSetupSecurityBloc extends MockBloc<SetupSecurityEvent, SetupSecurityState> implements SetupSecurityBloc {}

void main() {
  group('goldens', () {
    testGoldens('SetupSecuritySelectPinInProgress light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilder
          ..addScenario(
            name: '3 digits',
            widget: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
              MockSetupSecurityBloc(),
              const SetupSecuritySelectPinInProgress(3),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'in_progress.light');
    });

    testGoldens('SetupSecuritySelectPinFailed (sequentialDigits) light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilder
          ..addScenario(
            name: 'error state',
            widget: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
              MockSetupSecurityBloc(),
              const SetupSecuritySelectPinFailed(reason: PinValidationError.sequentialDigits),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'error.light');
    });

    testGoldens('SetupSecurityPinConfirmationFailed retry NOT allowed light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilder
          ..addScenario(
            name: 'setup failed',
            widget: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
              MockSetupSecurityBloc(),
              const SetupSecurityPinConfirmationFailed(retryAllowed: false),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'pin_confirmation_failed.light');
    });

    testGoldens('SetupSecurityPinConfirmationInProgress light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilder
          ..addScenario(
            widget: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
              MockSetupSecurityBloc(),
              const SetupSecurityPinConfirmationInProgress(6),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'pin_confirmation_in_progress.light');
    });

    testGoldens('SetupSecurityCompleted light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilder
          ..addScenario(
            widget: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
              MockSetupSecurityBloc(),
              SetupSecurityCompleted(),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'completed.light');
    });

    testGoldens('SetupSecurityPinConfirmationFailed retry allowed dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilder
          ..addScenario(
            widget: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
              MockSetupSecurityBloc(),
              const SetupSecurityPinConfirmationFailed(retryAllowed: true),
            ),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'pin_confirmation_failed.dark');
    });

    testGoldens('SetupSecurityCreatingWallet light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilder
          ..addScenario(
            widget: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
              MockSetupSecurityBloc(),
              SetupSecurityCreatingWallet(),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'creating_wallet.light');
    });

    testGoldens('SetupSecuritySelectPinFailed (tooFewUniqueDigits) dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilder
          ..addScenario(
            name: 'error state',
            widget: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
              MockSetupSecurityBloc(),
              const SetupSecuritySelectPinFailed(reason: PinValidationError.tooFewUniqueDigits),
            ),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'error.dark');
    });
  });
}
