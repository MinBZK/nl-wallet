import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/wallet/personalize/wallet_personalize_no_digid_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../util/device_utils.dart';
import '../../../util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('WalletPersonalizeNoDigidScreen Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeNoDigidScreen(),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'wallet_personalize_no_digid/light');
    });

    testGoldens('WalletPersonalizeNoDigidScreen Dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const WalletPersonalizeNoDigidScreen(),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'wallet_personalize_no_digid/dark');
    });
  });

  group('widgets', () {
    testWidgets('description is shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const WalletPersonalizeNoDigidScreen());
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.walletPersonalizeNoDigidPageDescription), findsOneWidget);
    });
  });
}
