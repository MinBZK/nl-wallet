import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/button/secondary_button.dart';
import 'package:wallet/src/feature/error/invariant/invariant_error_details_sheet.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

const kMockErrorCode = 'PVW-5915-mock-error-code';
const kGoldenSize = Size(350, 390);
const kReleaseGoldenSize = Size(350, 330);

void main() {
  group('goldens', () {
    testGoldens('light (debug build, with copy action)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const InvariantErrorDetailsSheet(code: kMockErrorCode),
        surfaceSize: kGoldenSize,
      );
      await screenMatchesGolden('invariant_error_details_sheet/light');
    });

    testGoldens('dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const InvariantErrorDetailsSheet(code: kMockErrorCode),
        surfaceSize: kGoldenSize,
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('invariant_error_details_sheet/dark');
    });

    testGoldens('light (non-debug build, without copy action)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const InvariantErrorDetailsSheet(code: kMockErrorCode, showCopyButton: false),
        surfaceSize: kReleaseGoldenSize,
      );
      await screenMatchesGolden('invariant_error_details_sheet/light_non_debug');
    });
  });

  group('widgets', () {
    testWidgets('shows the error code', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const InvariantErrorDetailsSheet(code: kMockErrorCode),
        surfaceSize: kGoldenSize,
      );

      expect(find.textContaining(kMockErrorCode), findsOneWidget);
    });

    testWidgets('shows the developer copy action in debug builds', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        const InvariantErrorDetailsSheet(code: kMockErrorCode),
        surfaceSize: kGoldenSize,
      );

      expect(find.text(l10n.invariantErrorDetailsSheetTitle, findRichText: true), findsOneWidget);
      // The "Copy" button is developer-only and only present in debug builds (which tests run in).
      expect(find.byType(SecondaryButton), findsOneWidget);
    });

    testWidgets('hides the copy action in non-debug builds', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const InvariantErrorDetailsSheet(code: kMockErrorCode, showCopyButton: false),
        surfaceSize: kReleaseGoldenSize,
      );

      expect(find.byType(SecondaryButton), findsNothing);
    });

    testWidgets('copies the error code to the clipboard when copy is tapped', (tester) async {
      MethodCall? clipboardCall;
      tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.platform, (call) async {
        if (call.method == 'Clipboard.setData') clipboardCall = call;
        return null;
      });
      addTearDown(
        () => tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.platform, null),
      );

      await tester.pumpWidgetWithAppWrapper(
        const InvariantErrorDetailsSheet(code: kMockErrorCode),
        surfaceSize: kGoldenSize,
      );

      await tester.tap(find.byType(SecondaryButton));
      await tester.pump();

      expect(clipboardCall, isNotNull);
      expect((clipboardCall!.arguments as Map)['text'], kMockErrorCode);
    });
  });
}
