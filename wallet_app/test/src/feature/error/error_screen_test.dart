import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/error/error_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';

void main() {
  DeviceBuilder deviceBuilder(WidgetTester tester) {
    return DeviceUtils.accessibilityDeviceBuilder
      ..addScenario(
        widget: ErrorScreen(
          title: 'Title',
          headline: 'Headline',
          description: 'Description',
          primaryActionText: 'Primary',
          onPrimaryActionPressed: () {},
          secondaryActionText: 'Secondary',
          onSecondaryActionPressed: () {},
        ),
        name: 'error_screen',
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

  testWidgets('ErrorScreen renders expected widgets', (tester) async {
    await tester.pumpWidget(
      WalletAppTestWidget(
        child: ErrorScreen(
          title: 'T',
          description: 'D',
          headline: 'H',
          primaryActionText: 'P',
          onPrimaryActionPressed: () {},
          secondaryActionText: 'S',
          onSecondaryActionPressed: () {},
        ),
      ),
    );

    // Setup finders
    final titleFinder = find.text('T');
    final descriptionFinder = find.text('D');
    final headlineFinder = find.text('H');
    final primaryActionFinder = find.text('P');
    final secondaryActionFinder = find.text('S');

    // Verify all expected widgets show up once
    expect(titleFinder, findsOneWidget);
    expect(descriptionFinder, findsOneWidget);
    expect(headlineFinder, findsOneWidget);
    expect(primaryActionFinder, findsOneWidget);
    expect(secondaryActionFinder, findsOneWidget);
  });
}
