import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/root/root_detected_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';
import '../../util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('WalletPersonalizeNoDigidScreen Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const RootDetectedScreen(),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'light');
    });

    testGoldens('WalletPersonalizeNoDigidScreen Dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const RootDetectedScreen(),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'dark');
    });
  });

  group('widgets', () {
    testWidgets('description is shown in paragraphs', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const RootDetectedScreen());
      final l10n = await TestUtils.englishLocalizations;
      l10n.rootDetectedScreenDescription.split('\n\n').forEach(
        (paragraph) {
          expect(find.text(paragraph), findsOneWidget);
        },
      );
    });
  });
}
