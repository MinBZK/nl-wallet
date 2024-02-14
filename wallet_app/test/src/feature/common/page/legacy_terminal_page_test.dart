import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/page/legacy_terminal_page.dart';

import '../../../../wallet_app_test_widget.dart';

void main() {
  group('goldens', () {
    testGoldens(
      'light legacy terminal page',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          LegacyTerminalPage(
            icon: Icons.add_card_outlined,
            title: 'Title',
            description: 'Description',
            primaryButtonCta: 'Close CTA',
            secondaryButtonCta: 'Secondary CTA',
            tertiaryButtonCta: 'Tertiary CTA',
            onPrimaryPressed: () {},
          ),
        );
        await screenMatchesGolden(tester, 'legacy_terminal_page/light');
      },
    );
    testGoldens(
      'dark legacy terminal page',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          LegacyTerminalPage(
            icon: Icons.add_card_outlined,
            title: 'Title',
            description: 'Description',
            primaryButtonCta: 'Close CTA',
            secondaryButtonCta: 'Secondary CTA',
            tertiaryButtonCta: 'Tertiary CTA',
            onPrimaryPressed: () {},
          ),
          brightness: Brightness.dark,
        );
        await screenMatchesGolden(tester, 'legacy_terminal_page/dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('widgets are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        LegacyTerminalPage(
          icon: Icons.add_card_outlined,
          title: 'T',
          description: 'D',
          primaryButtonCta: 'C',
          content: const Text('CC'),
          secondaryButtonCta: 'SBC',
          tertiaryButtonCta: 'TBC',
          onPrimaryPressed: () {},
        ),
      );

      // Validate that the widget exists
      final titleFinder = find.text('T');
      final descriptionFinder = find.text('D');
      final closeButtonFinder = find.text('C');
      final customContentFinder = find.text('CC');
      final secondaryButtonFinder = find.text('SBC');
      final tertiaryButtonFinder = find.text('TBC');
      expect(titleFinder, findsOneWidget);
      expect(descriptionFinder, findsOneWidget);
      expect(closeButtonFinder, findsOneWidget);
      expect(customContentFinder, findsOneWidget);
      expect(secondaryButtonFinder, findsOneWidget);
      expect(tertiaryButtonFinder, findsOneWidget);
    });

    testWidgets('close cta works', (tester) async {
      bool closeCalled = false;
      bool secondaryCalled = false;
      bool tertiaryCalled = false;
      await tester.pumpWidgetWithAppWrapper(
        LegacyTerminalPage(
          icon: Icons.add_card_outlined,
          title: 'T',
          description: 'D',
          primaryButtonCta: 'C',
          onPrimaryPressed: () => closeCalled = true,
          secondaryButtonCta: 'SB',
          tertiaryButtonCta: 'TB',
          onSecondaryButtonPressed: () => secondaryCalled = true,
          onTertiaryButtonPressed: () => tertiaryCalled = true,
        ),
      );

      // Validate that the widget exists
      final closeButtonFinder = find.text('C');
      final secondaryButtonFinder = find.text('SB');
      final tertiaryButtonFinder = find.text('TB');
      await tester.tap(closeButtonFinder);
      await tester.tap(secondaryButtonFinder);
      await tester.tap(tertiaryButtonFinder);
      expect(closeCalled, isTrue);
      expect(secondaryCalled, isTrue);
      expect(tertiaryCalled, isTrue);
    });
  });
}
