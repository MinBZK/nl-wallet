import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/menu_item.dart';
import 'package:wallet/src/feature/help/help_subcategory_screen.dart';
import 'package:wallet/src/navigation/wallet_routes.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mock_data.dart';
import '../../test_util/golden_utils.dart';

void main() {
  group('widgets', () {
    testWidgets('renders subcategory title and a MenuItem for every topic', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        HelpSubcategoryScreen(subcategory: WalletMockData.helpIntroductionSubcategory),
      );

      // Title appears in the app bar and as the hero heading — just assert >=1.
      expect(find.text(WalletMockData.helpIntroductionSubcategory.title), findsWidgets);

      final expectedTopics = WalletMockData.helpIntroductionSubcategory.groups
          .expand((g) => g.topics)
          .map((t) => t.title)
          .toList();
      for (final title in expectedTopics) {
        expect(find.text(title), findsOneWidget);
      }
      expect(find.byType(MenuItem), findsNWidgets(expectedTopics.length));
    });

    testWidgets('renders localized group headings for each topic group', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        HelpSubcategoryScreen(subcategory: WalletMockData.helpIntroductionSubcategory),
      );

      // The introduction fixture has both a 'help' and an 'information' group.
      expect(find.text('Help'), findsOneWidget);
      expect(find.text('Information'), findsOneWidget);
    });

    testWidgets('tapping a topic pushes the helpTopicRoute', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        HelpSubcategoryScreen(subcategory: WalletMockData.helpIntroductionSubcategory),
      );

      final firstTopic = WalletMockData.helpIntroductionSubcategory.groups.first.topics.first;
      await tester.tap(find.text(firstTopic.title));
      await tester.pumpAndSettle();

      // The test wrapper's onUnknownRoute renders the pushed route name (invisibly).
      expect(find.text(WalletRoutes.helpTopicRoute), findsOneWidget);
    });
  });

  group('goldens', () {
    testGoldens('HelpSubcategoryScreen light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        HelpSubcategoryScreen(subcategory: WalletMockData.helpIntroductionSubcategory),
      );
      await screenMatchesGolden('subcategory.light');
    });

    testGoldens('HelpSubcategoryScreen dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        HelpSubcategoryScreen(subcategory: WalletMockData.helpIntroductionSubcategory),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('subcategory.dark');
    });

    testGoldens('HelpSubcategoryScreen light - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        HelpSubcategoryScreen(subcategory: WalletMockData.helpIntroductionSubcategory),
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('subcategory.light.landscape');
    });
  });
}
