import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/menu_item.dart';
import 'package:wallet/src/feature/help/help_category_screen.dart';
import 'package:wallet/src/navigation/wallet_routes.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mock_data.dart';
import '../../test_util/golden_utils.dart';

void main() {
  group('widgets', () {
    testWidgets('renders a MenuItem for every subcategory', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        HelpCategoryScreen(category: WalletMockData.helpGettingStartedCategory),
      );

      for (final sub in WalletMockData.helpGettingStartedCategory.subcategories) {
        expect(find.text(sub.title), findsOneWidget);
      }
      expect(find.byType(MenuItem), findsNWidgets(WalletMockData.helpGettingStartedCategory.subcategories.length));
    });

    testWidgets('renders the category title in app bar and hero', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        HelpCategoryScreen(category: WalletMockData.helpGettingStartedCategory),
      );

      // App bar + body hero both use TitleText — expect at least one instance.
      expect(find.text(WalletMockData.helpGettingStartedCategory.title), findsWidgets);
    });

    testWidgets('tapping a subcategory pushes helpSubcategoryRoute', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        HelpCategoryScreen(category: WalletMockData.helpGettingStartedCategory),
      );

      await tester.tap(find.text(WalletMockData.helpGettingStartedCategory.subcategories.first.title));
      await tester.pumpAndSettle();

      expect(find.text(WalletRoutes.helpSubcategoryRoute), findsOneWidget);
    });
  });

  group('goldens', () {
    testGoldens('HelpCategoryScreen light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        HelpCategoryScreen(category: WalletMockData.helpGettingStartedCategory),
      );
      await screenMatchesGolden('category.light');
    });

    testGoldens('HelpCategoryScreen dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        HelpCategoryScreen(category: WalletMockData.helpGettingStartedCategory),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('category.dark');
    });

    testGoldens('HelpCategoryScreen light - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        HelpCategoryScreen(category: WalletMockData.helpGettingStartedCategory),
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('category.light.landscape');
    });
  });
}
