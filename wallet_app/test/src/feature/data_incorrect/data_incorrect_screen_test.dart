import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/data_incorrect/data_incorrect_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';
import '../../util/test_utils.dart';

void main() {
  group('goldens', () {
    DeviceBuilder deviceBuilder(WidgetTester tester) {
      return DeviceUtils.deviceBuilderWithPrimaryScrollController
        ..addScenario(
          widget: const DataIncorrectScreen(),
        );
    }

    testGoldens('DataIncorrectScreen light', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'light');
    });

    testGoldens('DataIncorrectScreen dark', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'dark');
    });
  });

  group('widgets', () {
    testWidgets('data screen renders as expected', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const DataIncorrectScreen());
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.dataIncorrectScreenHeaderTitle), findsAtLeast(1));
      expect(find.text(l10n.dataIncorrectScreenHeaderDescription), findsOneWidget);
      // Accept and decline CTAs are visible
      expect(find.text(l10n.dataIncorrectScreenDeclineCta), findsNWidgets(2 /* title & cta */));
      expect(find.text(l10n.dataIncorrectScreenApproveCta), findsOneWidget);
    });
  });
}
