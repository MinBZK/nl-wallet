import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/report_issue/report_issue_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('ReportIssueScreen light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ReportIssueScreen(options: ReportingOption.values),
      );
      await screenMatchesGolden('light');
    });

    testGoldens('ReportIssueScreen dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ReportIssueScreen(options: ReportingOption.values),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('dark');
    });
  });

  group('widgets', () {
    testWidgets('title and back button are shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const ReportIssueScreen(options: ReportingOption.values));

      final l10n = await TestUtils.englishLocalizations;
      final titleFinder = find.text(l10n.reportIssueScreenTitle);
      final backButtonFinder = find.text(l10n.generalBottomBackCta);
      expect(titleFinder, findsNWidgets(2) /* app bar + content */);
      expect(backButtonFinder, findsOneWidget);
    });
  });
}
