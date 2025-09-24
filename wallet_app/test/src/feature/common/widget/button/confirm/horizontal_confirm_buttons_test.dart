import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/button/confirm/horizontal_confirm_buttons.dart';
import 'package:wallet/src/feature/common/widget/button/primary_button.dart';
import 'package:wallet/src/feature/common/widget/button/secondary_button.dart';

import '../../../../../../wallet_app_test_widget.dart';

void main() {
  group('widgets', () {
    testWidgets('primary and secondary buttons are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HorizontalConfirmButtons(
          primaryButton: PrimaryButton(
            text: Text('A'),
          ),
          secondaryButton: SecondaryButton(
            text: Text('D'),
          ),
        ),
      );

      // Validate that both buttons exists
      final acceptButtonFinder = find.text('A');
      final declineButtonFinder = find.text('D');
      expect(acceptButtonFinder, findsOneWidget);
      expect(declineButtonFinder, findsOneWidget);
    });

    testWidgets('secondary button is off screen when hidden', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HorizontalConfirmButtons(
          primaryButton: PrimaryButton(
            text: Text('A'),
          ),
          secondaryButton: SecondaryButton(
            text: Text('D'),
          ),
          hideSecondaryButton: true,
        ),
      );

      const hiddenXAlignment =
          HorizontalConfirmButtons.kHiddenXAlignment; // The alignment to draw the button off-screen
      await tester.pumpAndSettle();

      final secondaryButtonFinder = find.text('D');
      final alignment = _getAlignmentThroughParent(tester, secondaryButtonFinder);
      expect(alignment.x, equals(hiddenXAlignment));
    });

    testWidgets('secondary button is visible when not hidden', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HorizontalConfirmButtons(
          primaryButton: PrimaryButton(
            text: Text('A'),
          ),
          secondaryButton: SecondaryButton(
            text: Text('D'),
          ),
          hideSecondaryButton: false,
        ),
      );

      const hiddenXAlignment = -3; // The alignment to draw the button off-screen
      await tester.pumpAndSettle();

      final secondaryButtonFinder = find.text('D');
      final alignment = _getAlignmentThroughParent(tester, secondaryButtonFinder);
      expect(alignment.x != hiddenXAlignment, isTrue);
    });

    testWidgets('secondary button show sup in semantics tree', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HorizontalConfirmButtons(
          primaryButton: PrimaryButton(
            text: Text('A'),
          ),
          secondaryButton: SecondaryButton(
            text: Text('D'),
          ),
          hideSecondaryButton: false,
        ),
      );

      await tester.pumpAndSettle();

      final secondaryButtonFinder = find.text('D');
      final isExcluding = _getExcludeSemanticsThroughParent(tester, secondaryButtonFinder);
      expect(isExcluding, isFalse);
    });

    testWidgets('secondary button does not show up in semantics tree when hidden', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HorizontalConfirmButtons(
          primaryButton: PrimaryButton(
            text: Text('A'),
          ),
          secondaryButton: SecondaryButton(
            text: Text('D'),
          ),
          hideSecondaryButton: true,
        ),
      );

      await tester.pumpAndSettle();

      final secondaryButtonFinder = find.text('D');
      final isExcluding = _getExcludeSemanticsThroughParent(tester, secondaryButtonFinder);
      expect(isExcluding, isTrue);
    });
  });
}

Alignment _getAlignmentThroughParent(WidgetTester tester, Finder finder) {
  return tester
          .widget<Align>(
            find.ancestor(
              of: finder,
              matching: find.byKey(HorizontalConfirmButtons.secondaryButtonAlignmentKey),
            ),
          )
          .alignment
      as Alignment;
}

bool _getExcludeSemanticsThroughParent(WidgetTester tester, Finder finder) {
  return tester
      .firstWidget<ExcludeSemantics>(
        find.ancestor(
          of: finder,
          matching: find.byType(ExcludeSemantics),
        ),
      )
      .excluding;
}
