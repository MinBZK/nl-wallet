import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/dialog/delete_card_dialog.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/dialog_utils.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

void main() {
  const cardTitle = 'Driving License';

  testWidgets('DeleteCardDialog shows expected copy', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      const DeleteCardDialog(cardTitle: cardTitle),
    );

    final l10n = await TestUtils.englishLocalizations;

    final titleFinder = find.text(l10n.deleteCardDialogTitle(cardTitle), findRichText: true);
    final bodyFinder = find.text(l10n.deleteCardDialogBody, findRichText: true);
    final cancelCtaFinder = find.text(l10n.deleteCardDialogCancelCta, findRichText: true);
    final confirmCtaFinder = find.text(l10n.deleteCardDialogConfirmCta, findRichText: true);

    expect(titleFinder, findsOneWidget);
    expect(bodyFinder, findsOneWidget);
    expect(cancelCtaFinder, findsOneWidget);
    expect(confirmCtaFinder, findsOneWidget);
  });

  testWidgets('DeleteCardDialog returns true when confirm is pressed', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      const DeleteCardDialog(cardTitle: cardTitle),
    );

    final l10n = await TestUtils.englishLocalizations;
    final confirmFinder = find.text(l10n.deleteCardDialogConfirmCta, findRichText: true);
    await tester.tap(confirmFinder);
  });

  testWidgets('DeleteCardDialog returns false when cancel is pressed', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      const DeleteCardDialog(cardTitle: cardTitle),
    );

    final l10n = await TestUtils.englishLocalizations;
    final cancelFinder = find.text(l10n.deleteCardDialogCancelCta, findRichText: true);
    await tester.tap(cancelFinder);
  });

  group('goldens', () {
    testGoldens('DeleteCardDialog', (tester) async {
      await DialogUtils.pumpDialog(
        tester,
        (context) => DeleteCardDialog.show(context, cardTitle: cardTitle),
      );
      await screenMatchesGolden('delete_card_dialog');
    });
  });
}
