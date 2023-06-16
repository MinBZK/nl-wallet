import 'dart:ui';

import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/pin_blocked/pin_blocked_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';

void main() {
  DeviceBuilder deviceBuilder(WidgetTester tester) {
    return DeviceUtils.accessibilityDeviceBuilder
      ..addScenario(
        widget: const PinBlockedScreen(),
        name: 'pin_blocked_screen',
      );
  }

  group('Golden Tests', () {
    testGoldens('Accessibility Light Test', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'accessibility_light');
    });

    testGoldens('Accessibility Dark Test', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'accessibility_dark');
    });
  });

  testWidgets('verify PinBlockedScreen renders expected text', (tester) async {
    await tester.pumpWidget(const WalletAppTestWidget(child: PinBlockedScreen()));

    final AppLocalizations locale = await AppLocalizations.delegate.load(const Locale('en'));
    final titleFinder = find.textContaining(locale.pinBlockedScreenTitle, findRichText: true);
    final headlineFinder = find.textContaining(locale.pinBlockedScreenHeadline, findRichText: true);
    final descriptionFinder = find.textContaining(locale.pinBlockedScreenDescription, findRichText: true);
    final ctaFinder = find.textContaining(locale.pinBlockedScreenResetWalletCta, findRichText: true);

    expect(titleFinder, findsOneWidget);
    expect(headlineFinder, findsOneWidget);
    expect(descriptionFinder, findsOneWidget);
    expect(ctaFinder, findsOneWidget);
  });
}
