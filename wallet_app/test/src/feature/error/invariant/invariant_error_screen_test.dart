import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/button/tertiary_button.dart';
import 'package:wallet/src/feature/error/invariant/invariant_error_details_sheet.dart';
import 'package:wallet/src/feature/error/invariant/invariant_error_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

const kMockErrorCode = 'PVW-5915-mock-error-code';

void main() {
  group('goldens', () {
    testGoldens('light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const InvariantErrorScreen(code: kMockErrorCode),
      );
      await screenMatchesGolden('invariant_error/light');
    });

    testGoldens('dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const InvariantErrorScreen(code: kMockErrorCode),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('invariant_error/dark');
    });

    testGoldens('landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const InvariantErrorScreen(code: kMockErrorCode),
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('invariant_error/landscape');
    });

    testGoldens('scaled', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const InvariantErrorScreen(code: kMockErrorCode),
        textScaleSize: 2,
      );
      await screenMatchesGolden('invariant_error/scaled');
    });
  });

  group('widgets', () {
    testWidgets('renders the title, sections and call-to-actions', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(const InvariantErrorScreen(code: kMockErrorCode));

      expect(find.text(l10n.invariantErrorScreenTitle, findRichText: true), findsWidgets);
      expect(find.text(l10n.invariantErrorScreenWhatCanYouDoTitle, findRichText: true), findsOneWidget);
      expect(find.text(l10n.invariantErrorScreenStillNotWorkingTitle, findRichText: true), findsOneWidget);
      expect(find.text(l10n.invariantErrorScreenSeeDetailsCta, findRichText: true), findsOneWidget);
      expect(find.text(l10n.invariantErrorScreenStartAgainCta, findRichText: true), findsOneWidget);
    });

    testWidgets('tapping "See details" shows the details sheet', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const InvariantErrorScreen(code: kMockErrorCode));

      await tester.tap(find.byType(TertiaryButton));
      await tester.pumpAndSettle();

      expect(find.byType(InvariantErrorDetailsSheet), findsOneWidget);
    });
  });
}
