import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/screen/placeholder_screen.dart';

import '../../../../wallet_app_test_widget.dart';

void main() {
  group('goldens', () {
    testGoldens(
      'light generic placeholder',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const PlaceholderScreen(type: PlaceholderType.generic),
        );
        await screenMatchesGolden(tester, 'placeholder_screen/light.generic');
      },
    );
    testGoldens(
      'dark generic placeholder',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const PlaceholderScreen(type: PlaceholderType.generic),
          brightness: Brightness.dark,
        );
        await screenMatchesGolden(tester, 'placeholder_screen/dark.generic');
      },
    );
    testGoldens(
      'light contract placeholder',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const PlaceholderScreen(type: PlaceholderType.contract),
        );
        await screenMatchesGolden(tester, 'placeholder_screen/light.contract');
      },
    );
    testGoldens(
      'light generic placeholder',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(const PlaceholderScreen(type: PlaceholderType.generic),
            surfaceSize: const Size(812, 375));
        await screenMatchesGolden(tester, 'placeholder_screen/light.generic.landscape');
      },
    );
  });
}
