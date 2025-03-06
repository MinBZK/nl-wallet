import 'dart:ui';

import 'package:wallet/l10n/generated/app_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/pin_blocked/pin_blocked_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';
import '../../util/test_utils.dart';

void main() {
  DeviceBuilder deviceBuilder(WidgetTester tester) {
    return DeviceUtils.deviceBuilderWithPrimaryScrollController
      ..addScenario(
        widget: const PinBlockedScreen(),
        name: 'pin_blocked_screen',
      );
  }

  group('goldens', () {
    testGoldens('PinBlockedScreen light', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'light');
    });

    testGoldens('PinBlockedScreen dark', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'dark');
    });
  });

  group('widgets', () {
    testWidgets('verify PinBlockedScreen renders expected text', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const PinBlockedScreen());

      final AppLocalizations l10n = await TestUtils.englishLocalizations;
      final headlineFinder = find.textContaining(l10n.pinBlockedScreenHeadline, findRichText: true);
      final descriptionFinder = find.textContaining(l10n.pinBlockedScreenDescription, findRichText: true);
      final ctaFinder = find.textContaining(l10n.pinBlockedScreenResetWalletCta, findRichText: true);

      expect(headlineFinder, findsNWidgets(2) /*In content and appbar*/);
      expect(descriptionFinder, findsOneWidget);
      expect(ctaFinder, findsOneWidget);
    });
  });
}
