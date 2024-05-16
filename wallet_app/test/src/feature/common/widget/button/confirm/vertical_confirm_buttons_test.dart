import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/button/confirm/vertical_confirm_buttons.dart';
import 'package:wallet/src/feature/common/widget/button/primary_button.dart';
import 'package:wallet/src/feature/common/widget/button/secondary_button.dart';

import '../../../../../../wallet_app_test_widget.dart';

void main() {
  group('widgets', () {
    testWidgets('primary and secondary buttons are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const VerticalConfirmButtons(
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

    testWidgets('secondary button is invisible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const VerticalConfirmButtons(
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

      final declineButtonFinder = find.text('D');
      // Validate button is invisible
      final opacity = _getOpacityThroughParent(tester, declineButtonFinder);
      expect(opacity, equals(0));
    });
  });
}

double _getOpacityThroughParent(WidgetTester tester, Finder finder) {
  return tester
      .widget<Opacity>(
        find.ancestor(
          of: finder,
          matching: find.byType(Opacity),
        ),
      )
      .opacity;
}
