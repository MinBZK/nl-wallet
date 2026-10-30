import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/sheet/wallet_modal_bottom_sheet.dart';

import '../../../../wallet_app_test_widget.dart';

void main() {
  Future<void> pumpAndShowSheet(WidgetTester tester, {bool isDismissible = true}) async {
    await tester.pumpWidgetWithAppWrapper(
      Scaffold(
        body: Builder(
          builder: (context) {
            return Align(
              alignment: Alignment.topCenter,
              child: TextButton(
                key: const Key('showSheet'),
                onPressed: () => showWalletModalBottomSheet<void>(
                  context: context,
                  isDismissible: isDismissible,
                  builder: (context) => const SizedBox(height: 100, child: Text('Sheet content')),
                ),
                child: const Text('Show'),
              ),
            );
          },
        ),
      ),
    );
    await tester.tap(find.byKey(const Key('showSheet')));
    await tester.pumpAndSettle();
  }

  group('widgets', () {
    testWidgets('scrim is not exposed to assistive technology', (tester) async {
      final handle = tester.ensureSemantics();
      await pumpAndShowSheet(tester);

      final scrimLabel = MaterialLocalizations.of(tester.element(find.byKey(const Key('showSheet')))).scrimLabel;
      expect(find.text('Sheet content'), findsOneWidget);
      expect(find.bySemanticsLabel(scrimLabel), findsNothing);
      handle.dispose();
    });

    testWidgets('tapping the scrim dismisses the sheet', (tester) async {
      await pumpAndShowSheet(tester);

      await tester.tapAt(const Offset(10, 10));
      await tester.pumpAndSettle();

      expect(find.text('Sheet content'), findsNothing);
    });

    testWidgets('tapping the scrim does not dismiss a non-dismissible sheet', (tester) async {
      await pumpAndShowSheet(tester, isDismissible: false);

      await tester.tapAt(const Offset(10, 10));
      await tester.pumpAndSettle();

      expect(find.text('Sheet content'), findsOneWidget);
    });
  });
}
