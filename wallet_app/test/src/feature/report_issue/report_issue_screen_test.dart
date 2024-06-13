import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/report_issue/report_issue_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';
import '../../util/test_utils.dart';

void main() {
  group('goldens', () {
    DeviceBuilder deviceBuilder(WidgetTester tester) => DeviceUtils.deviceBuilderWithPrimaryScrollController;

    testGoldens('ReportIssueScreen light', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester)..addScenario(widget: const ReportIssueScreen(options: ReportingOption.values)),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'light');
    });

    testGoldens('ReportIssueScreen dark', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester)..addScenario(widget: const ReportIssueScreen(options: ReportingOption.values)),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'dark');
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
