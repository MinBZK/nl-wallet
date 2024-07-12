import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/digid_help/digid_help_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';
import '../../util/test_utils.dart';

void main() {
  group('goldens', () {
    DeviceBuilder deviceBuilder(WidgetTester tester) {
      return DeviceUtils.deviceBuilderWithPrimaryScrollController
        ..addScenario(
          widget: const DigidHelpScreen(),
        );
    }

    testGoldens('DigidHelpScreen light', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'light');
    });

    testGoldens('DigidHelpScreen dark', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'dark');
    });
  });

  group('widgets', () {
    testWidgets('digid help screen renders expected title and CTAs', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const DigidHelpScreen());
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.digidHelpScreenTitle), findsAtLeast(1));
      // Help CTAs are visible
      expect(find.text(l10n.digidHelpScreenNoDigidCta), findsOneWidget);
      expect(find.text(l10n.digidHelpScreenHelpNeededCta), findsOneWidget);
    });
  });
}
