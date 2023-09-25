import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/wallet/personalize/wallet_personalize_setup_failed_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../util/device_utils.dart';
import '../../../util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('WalletPersonalizeSetupFailedScreen Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeSetupFailedScreen(),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize_setup_failed/light');
    });

    testGoldens('WalletPersonalizeSetupFailedScreen Dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeSetupFailedScreen(),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'wallet_personalize_setup_failed/dark');
    });
  });

  group('widgets', () {
    testWidgets('description is shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const WalletPersonalizeSetupFailedScreen());
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.walletPersonalizeSetupFailedScreenDescription), findsOneWidget);
    });

    testWidgets('cta is shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const WalletPersonalizeSetupFailedScreen());
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.walletPersonalizeSetupFailedScreenCta), findsOneWidget);
    });
  });
}
