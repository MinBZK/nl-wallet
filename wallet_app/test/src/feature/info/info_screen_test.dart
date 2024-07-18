import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/info/info_screen.dart';

import '../../../wallet_app_test_widget.dart';

void main() {
  testWidgets('provided title and description are shown', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(const InfoScreen(title: 'title', description: 'description'));

    expect(find.text('title'), findsAtLeast(1));
    expect(find.text('description'), findsOneWidget);
  });
}
