import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/update/app_blocked_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';
import '../../util/test_utils.dart';

void main() {
  group('goldens', () {
    DeviceBuilder deviceBuilder(WidgetTester tester) {
      return DeviceUtils.deviceBuilderWithPrimaryScrollController..addScenario(widget: const AppBlockedScreen());
    }

    testGoldens('Light Test', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(),
      );
      await tester.pumpAndSettle();

      await screenMatchesGolden(tester, 'blocked.light');
    });

    testGoldens('Dark Test', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await tester.pumpAndSettle();

      await screenMatchesGolden(tester, 'blocked.dark');
    });
  });

  group('widgets', () {
    testWidgets('Title is shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AppBlockedScreen(),
      );

      final l10n = await TestUtils.englishLocalizations;

      final titleFinder = find.textContaining(l10n.appBlockedScreenTitle, findRichText: true);
      // Title rendered twice, in content and (hidden) in sliverappbar.
      expect(titleFinder, findsNWidgets(2));
    });
  });
}
