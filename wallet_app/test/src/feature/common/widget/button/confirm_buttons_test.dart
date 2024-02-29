import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/widget/button/confirm_buttons.dart';

import '../../../../../wallet_app_test_widget.dart';

void main() {
  const kGoldenSize = Size(350, 100);

  group('goldens', () {
    testGoldens(
      'confirm buttons light',
      (tester) async {
        await tester.pumpWidgetBuilder(
          ConfirmButtons(
            primaryText: 'accept',
            secondaryText: 'decline',
            onPrimaryPressed: () {},
            onSecondaryPressed: () {},
          ),
          wrapper: walletAppWrapper(),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'confirm_buttons/light');
      },
    );
    testGoldens(
      'confirm buttons with custom icons light',
      (tester) async {
        await tester.pumpWidgetBuilder(
          ConfirmButtons(
            primaryText: 'accept',
            secondaryText: 'decline',
            onPrimaryPressed: () {},
            onSecondaryPressed: () {},
            primaryIcon: Icons.language_outlined,
            secondaryIcon: Icons.add_card_outlined,
          ),
          wrapper: walletAppWrapper(),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'confirm_buttons/light.icons');
      },
    );
    testGoldens(
      'confirm buttons dark',
      (tester) async {
        await tester.pumpWidgetBuilder(
          ConfirmButtons(
            primaryText: 'accept',
            secondaryText: 'decline',
            onPrimaryPressed: () {},
            onSecondaryPressed: () {},
          ),
          wrapper: walletAppWrapper(brightness: Brightness.dark),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'confirm_buttons/dark');
      },
    );
    testGoldens(
      'confirm buttons stacked light',
      (tester) async {
        await tester.pumpWidgetBuilder(
          ConfirmButtons(
            primaryText: 'accept',
            secondaryText: 'decline',
            onPrimaryPressed: () {},
            onSecondaryPressed: () {},
            forceVertical: true,
          ),
          wrapper: walletAppWrapper(brightness: Brightness.light),
          surfaceSize: const Size(350, 160),
        );
        await screenMatchesGolden(tester, 'confirm_buttons/light.stacked.forced');
      },
    );
    testGoldens(
      'confirm buttons stacked light',
      (tester) async {
        await tester.pumpWidgetBuilder(
          ConfirmButtons(
            primaryText: 'accept',
            secondaryText: 'decline',
            onPrimaryPressed: () {},
            onSecondaryPressed: () {},
          ),
          wrapper: walletAppWrapper(brightness: Brightness.light),
          surfaceSize: const Size(156, 156),
        );
        await screenMatchesGolden(tester, 'confirm_buttons/light.stacked');
      },
    );
  });

  group('widgets', () {
    testWidgets('buttons are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        ConfirmButtons(
          primaryText: 'A',
          secondaryText: 'D',
          onPrimaryPressed: () {},
          onSecondaryPressed: () {},
        ),
      );

      // Validate that both buttons exists
      final acceptButtonFinder = find.text('A');
      final declineButtonFinder = find.text('D');
      expect(acceptButtonFinder, findsOneWidget);
      expect(declineButtonFinder, findsOneWidget);
    });

    testWidgets('build method should not fail when screenWidth is super narrow', (tester) async {
      // This test is introduced to verify a fix, as rendering the ConfirmButtons
      // on a very narrow screen led to negative numbers, causing the app to crash.
      await tester.pumpWidgetWithAppWrapper(
        ConfirmButtons(
          primaryText: 'A',
          secondaryText: 'D',
          onPrimaryPressed: () {},
          onSecondaryPressed: () {},
        ),
        surfaceSize: const Size(120, 300),
      );
    });
  });
}
