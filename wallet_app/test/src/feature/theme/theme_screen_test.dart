import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/theme/tabs/button_styles_tab.dart';
import 'package:wallet/src/feature/theme/tabs/color_styles_tab.dart';
import 'package:wallet/src/feature/theme/tabs/other_styles_tab.dart';
import 'package:wallet/src/feature/theme/tabs/text_styles_tab.dart';
import 'package:wallet/src/feature/theme/theme_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/test_utils.dart';

void main() {
  setUp(TestUtils.mockSensorsPlugin);

  group('verify all tabs can be rendered', () {
    testWidgets('button styles tab is shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const ThemeScreen());
      await tester.tap(find.text('Buttons'));
      await tester.pumpAndSettle();
      expect(find.byType(ButtonStylesTab), findsOneWidget);
    });

    testWidgets('text styles tab is shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const ThemeScreen());
      await tester.tap(find.text('TextStyles'));
      await tester.pumpAndSettle();
      expect(find.byType(TextStylesTab), findsOneWidget);
    });

    testWidgets('color styles tab is shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const ThemeScreen());
      await tester.tap(find.text('Colors'));
      await tester.pumpAndSettle();
      expect(find.byType(ColorStylesTab), findsOneWidget);
    });

    testWidgets('other styles tab is shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const ThemeScreen());
      await tester.tap(find.text('Other'));
      await tester.pumpAndSettle();
      expect(find.byType(OtherStylesTab), findsOneWidget);
    });
  });
}
