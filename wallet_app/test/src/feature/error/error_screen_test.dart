import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/button/primary_button.dart';
import 'package:wallet/src/feature/common/widget/button/tertiary_button.dart';
import 'package:wallet/src/feature/error/error_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('ErrorScreen light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        ErrorScreen(
          title: 'Headline',
          description: 'Description',
          primaryButton: PrimaryButton(
            text: const Text('Primary'),
            onPressed: () {},
          ),
          secondaryButton: TertiaryButton(
            text: const Text('Secondary'),
            onPressed: () {},
          ),
        ),
      );
      await screenMatchesGolden('light');
    });

    testGoldens('ErrorScreen dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        ErrorScreen(
          title: 'Headline',
          description: 'Description',
          primaryButton: PrimaryButton(
            text: const Text('Primary'),
            onPressed: () {},
          ),
          secondaryButton: TertiaryButton(
            text: const Text('Secondary'),
            onPressed: () {},
          ),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('dark');
    });

    testGoldens('ErrorScreen.showGeneric()', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            return ElevatedButton(
              onPressed: () => ErrorScreen.showGeneric(context, secured: false),
              child: const Text('generic'),
            );
          },
        ),
      );
      // Tap the button to open the generic error screen
      await tester.tap(find.text('generic'));
      await tester.pumpAndSettle();
      // Verify it's displayed correctly
      await screenMatchesGolden('generic.light');
    });

    testGoldens('ErrorScreen.showNetwork()', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            return ElevatedButton(
              onPressed: () => ErrorScreen.showNetwork(context, secured: false),
              child: const Text('network'),
            );
          },
        ),
      );
      // Tap the button to open the server error screen
      await tester.tap(find.text('network'));
      await tester.pumpAndSettle();
      // Verify it's displayed correctly
      await screenMatchesGolden('network.light');
    });

    testGoldens('ErrorScreen.showNoInternet()', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            return ElevatedButton(
              onPressed: () => ErrorScreen.showNoInternet(context, secured: false),
              child: const Text('no_internet'),
            );
          },
        ),
      );
      // Tap the button to open the server error screen
      await tester.tap(find.text('no_internet'));
      await tester.pumpAndSettle();
      // Verify it's displayed correctly
      await screenMatchesGolden('no_internet.light');
    });
  });

  group('widgets', () {
    testWidgets('ErrorScreen renders expected widgets', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        ErrorScreen(
          description: 'D',
          title: 'H',
          primaryButton: PrimaryButton(
            text: const Text('P'),
            onPressed: () {},
          ),
          secondaryButton: TertiaryButton(
            text: const Text('S'),
            onPressed: () {},
          ),
        ),
      );

      // Setup finders
      final descriptionFinder = find.text('D', findRichText: true);
      final headlineFinder = find.text('H', findRichText: true);
      final primaryActionFinder = find.text('P', findRichText: true);
      final secondaryActionFinder = find.text('S', findRichText: true);

      // Verify all expected widgets show up once
      expect(descriptionFinder, findsOneWidget);
      expect(headlineFinder, findsNWidgets(2) /* app bar + content */);
      expect(primaryActionFinder, findsOneWidget);
      expect(secondaryActionFinder, findsOneWidget);
    });
  });
}
