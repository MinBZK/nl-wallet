import 'dart:ui';

import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/pin_timeout/pin_timeout_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';

void main() {
  DeviceBuilder deviceBuilder(WidgetTester tester) {
    return DeviceUtils.accessibilityDeviceBuilder
      ..addScenario(
        widget: PinTimeoutScreen(
          expiryTime: DateTime.now().add(const Duration(seconds: 15)),
        ),
        name: 'pin_timeout_screen',
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

  testWidgets('verify PinTimeoutScreen renders expected text', (tester) async {
    await tester.pumpWidget(
      WalletAppTestWidget(
        child: PinTimeoutScreen(
          expiryTime: DateTime.now().add(const Duration(seconds: 5)),
        ),
      ),
    );

    final AppLocalizations locale = await AppLocalizations.delegate.load(const Locale('en'));
    final titleFinder = find.textContaining(locale.pinTimeoutScreenTitle, findRichText: true);
    final headlineFinder = find.textContaining(locale.pinTimeoutScreenHeadline, findRichText: true);
    final ctaFinder = find.textContaining(locale.pinTimeoutScreenForgotPinCta, findRichText: true);

    expect(titleFinder, findsOneWidget);
    expect(headlineFinder, findsOneWidget);
    expect(ctaFinder, findsOneWidget);
  });
}
