import 'package:flutter/cupertino.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/sign/page/sign_confirm_pin_page.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.dart';
import '../../../util/device_utils.dart';
import '../../../util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('SignConfirmPinPage light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: SignConfirmPinPage(
              bloc: PinBloc(Mocks.create()),
              onPinValidated: (_) {},
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'sign_confirm_pin/light');
    });

    testGoldens('SignConfirmPinPage dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: SignConfirmPinPage(
              bloc: PinBloc(Mocks.create()),
              onPinValidated: (_) {},
            ),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'sign_confirm_pin/dark');
    });
  });

  group('widgets', () {
    testWidgets('SignConfirmPinPage renders the correct title & subtitle', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        SignConfirmPinPage(
          onPinValidated: (_) {},
          bloc: PinBloc(Mocks.create()),
        ),
      );

      // Setup finders
      final titleFinder = find.text(l10n.signConfirmPinPageTitle);
      final descriptionFinder = find.text(l10n.signConfirmPinPageDescription);

      // Verify all expected widgets show up once
      expect(titleFinder, findsOneWidget);
      expect(descriptionFinder, findsOneWidget);
    });
  });
}
