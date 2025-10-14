import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/demo/demo_screen.dart';
import 'package:wallet/src/navigation/wallet_routes.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DemoScreen(),
      );

      await screenMatchesGolden('light');
    });

    testGoldens('dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DemoScreen(),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );

      await screenMatchesGolden('dark.landscape');
    });
  });

  group('widget', () {
    testWidgets('navigates to introduction when continue is pressed', (WidgetTester tester) async {
      // Given
      await tester.pumpWidgetWithAppWrapper(const DemoScreen());

      // When
      final l10n = await TestUtils.englishLocalizations;
      await tester.tap(find.text(l10n.demoScreenContinueCta));
      await tester.pumpAndSettle();

      // Then (test app should render placeholder with route name after successful navigation)
      expect(find.text(WalletRoutes.introductionRoute), findsOneWidget);
    });
  });
}
