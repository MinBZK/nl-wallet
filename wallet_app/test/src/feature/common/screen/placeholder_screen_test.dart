import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/screen/placeholder_screen.dart';

import '../../../../wallet_app_test_widget.dart';

void main() {
  group('widgets', () {
    testWidgets('Placeholder screen headline and description', (tester) async {
      await tester.pumpWidget(
        const WalletAppTestWidget(
          child: PlaceholderScreen(
            headline: 'H',
            description: 'D',
          ),
        ),
      );

      // Setup finders
      final headlineFinder = find.text('H');
      final descriptionFinder = find.text('D');

      // Verify all expected widgets show up once
      expect(headlineFinder, findsNWidgets(2) /* app bar + content */);
      expect(descriptionFinder, findsOneWidget);
    });
  });
}
