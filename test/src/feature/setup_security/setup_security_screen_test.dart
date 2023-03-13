import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/pin/pin_validation_error.dart';
import 'package:wallet/src/feature/setup_security/bloc/setup_security_bloc.dart';
import 'package:wallet/src/feature/setup_security/setup_security_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';

class MockSetupSecurityBloc extends MockBloc<SetupSecurityEvent, SetupSecurityState> implements SetupSecurityBloc {}

void main() {
  group('Golden Tests', () {
    testGoldens(
      'Accessibility Test',
      (tester) async {
        final deviceBuilder = DeviceUtils.accessibilityDeviceBuilder;
        deviceBuilder.addScenario(
          name: '3 digits',
          widget: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
            MockSetupSecurityBloc(),
            const SetupSecuritySelectPinInProgress(3),
          ),
        );
        deviceBuilder.addScenario(
          name: 'error state',
          widget: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
            MockSetupSecurityBloc(),
            const SetupSecuritySelectPinFailed(reason: PinValidationError.sequentialDigits),
          ),
        );
        deviceBuilder.addScenario(
          name: 'setup failed',
          widget: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
            MockSetupSecurityBloc(),
            const SetupSecurityPinConfirmationFailed(retryAllowed: false),
          ),
        );

        await tester.pumpDeviceBuilder(
          deviceBuilder,
          wrapper: walletAppWrapper(),
        );
        await screenMatchesGolden(tester, 'accessibility_scaling');
      },
    );
  });
}
