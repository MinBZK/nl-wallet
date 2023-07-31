import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/theme/theme_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/test_utils.dart';

void main() {
  const otherTabSize = Size(375, 3100);

  setUp(() => TestUtils.mockAccelerometerPlugin());

  group('goldens', () {
    testGoldens(
      'text styles light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(const ThemeScreen());
        await tester.tap(find.text('TextStyles'));
        await tester.pumpAndSettle();
        await screenMatchesGolden(tester, 'text_styles.light');
      },
    );
    testGoldens(
      'buttons light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(const ThemeScreen());
        await tester.tap(find.text('Buttons'));
        await tester.pumpAndSettle();
        await screenMatchesGolden(tester, 'buttons.light');
      },
    );
    testGoldens(
      'colors light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(const ThemeScreen());
        await tester.tap(find.text('Colors'));
        await tester.pumpAndSettle();
        await screenMatchesGolden(tester, 'colors.light');
      },
    );
    testGoldens(
      'other light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ThemeScreen(),
          surfaceSize: otherTabSize,
        );
        await tester.tap(find.text('Other'));
        await tester.pumpAndSettle();
        await screenMatchesGolden(tester, 'other.light');
      },
    );

    testGoldens(
      'text styles dark',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ThemeScreen(),
          brightness: Brightness.dark,
        );
        await tester.tap(find.text('TextStyles'));
        await tester.pumpAndSettle();
        await screenMatchesGolden(tester, 'text_styles.dark');
      },
    );
    testGoldens(
      'buttons dark',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ThemeScreen(),
          brightness: Brightness.dark,
        );
        await tester.tap(find.text('Buttons'));
        await tester.pumpAndSettle();
        await screenMatchesGolden(tester, 'buttons.dark');
      },
    );
    testGoldens(
      'colors dark',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ThemeScreen(),
          brightness: Brightness.dark,
        );
        await tester.tap(find.text('Colors'));
        await tester.pumpAndSettle();
        await screenMatchesGolden(tester, 'colors.dark');
      },
    );
    testGoldens(
      'other dark',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ThemeScreen(),
          brightness: Brightness.dark,
          surfaceSize: otherTabSize,
        );
        await tester.tap(find.text('Other'));
        await tester.pumpAndSettle();
        await screenMatchesGolden(tester, 'other.dark');
      },
    );
  });
}
