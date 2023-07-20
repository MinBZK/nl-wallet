import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/verification/page/verification_confirm_pin_page.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.dart';
import '../../../util/device_utils.dart';
import '../../../util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('VerificationConfirmPinPage light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: VerificationConfirmPinPage(
              bloc: PinBloc(Mocks.create()),
              onPinValidated: () {},
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'verification_confirm_pin/light');
    });

    testGoldens('VerificationConfirmPinPage dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: VerificationConfirmPinPage(
              bloc: PinBloc(Mocks.create()),
              onPinValidated: () {},
            ),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'verification_confirm_pin/dark');
    });
  });

  group('widgets', () {
    testWidgets('VerificationConfirmPinPage renders the correct title & subtitle', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidget(
        WalletAppTestWidget(
          child: VerificationConfirmPinPage(
            onPinValidated: () {},
            bloc: PinBloc(Mocks.create()),
          ),
        ),
      );

      // Setup finders
      final titleFinder = find.text(l10n.verificationConfirmPinPageTitle);
      final descriptionFinder = find.text(l10n.verificationConfirmPinPageDescription);

      // Verify all expected widgets show up once
      expect(titleFinder, findsOneWidget);
      expect(descriptionFinder, findsOneWidget);
    });
  });
}
