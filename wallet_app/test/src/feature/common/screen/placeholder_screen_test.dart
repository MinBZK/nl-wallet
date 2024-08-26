import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/screen/placeholder_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../util/test_utils.dart';

void main() {
  group('widgets', () {
    testWidgets('placeholder screen renders headline and description', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const PlaceholderScreen(
          headline: 'H',
          description: 'D',
        ),
      );

      // Setup finders
      final headlineFinder = find.text('H', findRichText: true);
      final descriptionFinder = find.text('D', findRichText: true);

      // Verify all expected widgets show up once
      expect(headlineFinder, findsNWidgets(2) /* app bar + content */);
      expect(descriptionFinder, findsOneWidget);
    });

    testWidgets('showGeneric shows the generic placeholder', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            return TextButton(
              child: const Text('generic'),
              onPressed: () => PlaceholderScreen.showGeneric(context, secured: false),
            );
          },
        ),
      );
      await tester.tap(find.text('generic', findRichText: true));
      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;
      // Expect generic placeholder copy
      expect(find.text(l10n.placeholderScreenHeadline, findRichText: true), findsAtLeast(1));
      expect(find.text(l10n.placeholderScreenGenericDescription, findRichText: true), findsOneWidget);
    });

    testWidgets('showHelp shows the help placeholder', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            return TextButton(
              child: const Text('help'),
              onPressed: () => PlaceholderScreen.showHelp(context, secured: false),
            );
          },
        ),
      );
      await tester.tap(find.text('help'));
      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;
      // Expect generic placeholder copy
      expect(find.text(l10n.placeholderScreenHelpHeadline, findRichText: true), findsAtLeast(1));
      expect(find.text(l10n.placeholderScreenHelpDescription, findRichText: true), findsOneWidget);
    });

    testWidgets('showContract shows the contract placeholder', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            return TextButton(
              child: const Text('contract'),
              onPressed: () => PlaceholderScreen.showContract(context, secured: false),
            );
          },
        ),
      );
      await tester.tap(find.text('contract', findRichText: true));
      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;
      // Expect generic placeholder copy
      expect(find.text(l10n.placeholderScreenHeadline, findRichText: true), findsAtLeast(1));
      expect(find.text(l10n.placeholderScreenContractDescription, findRichText: true), findsOneWidget);
    });
  });
}
