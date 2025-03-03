import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/feature/disclosure/widget/card_attribute_row.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';

void main() {
  testWidgets('card title is rendered', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      CardAttributeRow(
        entry: {WalletMockData.card: WalletMockData.card.attributes}.entries.first,
      ),
    );

    expect(find.textContaining(WalletMockData.card.title.testValue), findsOneWidget);
  });

  testWidgets('card attribute labels are rendered', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      CardAttributeRow(
        entry: {WalletMockData.card: WalletMockData.card.attributes}.entries.first,
      ),
    );

    for (final attribute in WalletMockData.card.attributes) {
      expect(find.text(attribute.label.testValue), findsOneWidget);
    }
  });
}
