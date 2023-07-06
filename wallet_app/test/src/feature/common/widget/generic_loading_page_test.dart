import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/widget/generic_loading_page.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../util/device_utils.dart';

void main() {
  DeviceBuilder deviceBuilder(WidgetTester tester) {
    return DeviceUtils.accessibilityDeviceBuilder
      ..addScenario(
        widget: GenericLoadingPage(
          title: 'Title',
          description: 'Description',
          onCancel: () {},
          cancelCta: 'Cancel',
        ),
        name: 'generic_loading_page',
      );
  }

  group('Golden Tests', () {
    testGoldens('Accessibility Light Test', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'generic_loading_page/accessibility_light');
    });

    testGoldens('Accessibility Dark Test', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'generic_loading_page/accessibility_dark');
    });
  });

  testWidgets('GenericLoadingPage renders expected widgets without cancelButton', (tester) async {
    await tester.pumpWidget(
      const WalletAppTestWidget(
        child: GenericLoadingPage(
          title: 'T',
          description: 'D',
          cancelCta: 'C',
        ),
      ),
    );

    // Setup finders
    final titleFinder = find.text('T');
    final descriptionFinder = find.text('D');
    final cancelButtonFinder = find.text('C');

    // Verify all expected widgets show up once
    expect(titleFinder, findsOneWidget);
    expect(descriptionFinder, findsOneWidget);
    expect(cancelButtonFinder, findsNothing);
  });

  testWidgets('GenericLoadingPage renders expected widgets with cancelButton', (tester) async {
    await tester.pumpWidget(
      WalletAppTestWidget(
        child: GenericLoadingPage(
          title: 'T',
          description: 'D',
          cancelCta: 'C',
          onCancel: () {},
        ),
      ),
    );

    // Setup finders
    final titleFinder = find.text('T');
    final descriptionFinder = find.text('D');
    final cancelButtonFinder = find.text('C');

    // Verify all expected widgets show up once
    expect(titleFinder, findsOneWidget);
    expect(descriptionFinder, findsOneWidget);
    expect(cancelButtonFinder, findsOneWidget);
  });
}
