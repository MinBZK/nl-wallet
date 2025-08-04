import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/card/data/card_data_incorrect_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/test_utils.dart';

void main() {
  testWidgets('screen renders with expected title and body', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(const CardDataIncorrectScreen());
    final l10n = await TestUtils.englishLocalizations;

    expect(find.text(l10n.cardDataIncorrectScreenSubhead), findsOneWidget);

    l10n.cardDataIncorrectScreenDescription.split('\n\n').forEach(
      (paragraph) {
        expect(find.text(paragraph), findsOneWidget);
      },
    );
  });
}
