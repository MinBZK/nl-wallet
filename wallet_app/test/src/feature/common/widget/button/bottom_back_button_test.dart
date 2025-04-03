import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/button/bottom_back_button.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../test_util/test_utils.dart';

void main() {
  group('widgets', () {
    testWidgets('buttons uses generic back button text', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const BottomBackButton());

      final l10n = await TestUtils.englishLocalizations;
      final textFinder = find.text(l10n.generalBottomBackCta);
      expect(textFinder, findsOneWidget);
    });

    testWidgets('expected back icon is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const BottomBackButton());

      final iconFinder = find.byIcon(Icons.arrow_back);
      expect(iconFinder, findsOneWidget);
    });
  });
}
