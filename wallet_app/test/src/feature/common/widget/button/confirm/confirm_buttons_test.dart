import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/button/confirm/confirm_buttons.dart';
import 'package:wallet/src/feature/common/widget/button/confirm/horizontal_confirm_buttons.dart';
import 'package:wallet/src/feature/common/widget/button/confirm/vertical_confirm_buttons.dart';
import 'package:wallet/src/feature/common/widget/button/primary_button.dart';
import 'package:wallet/src/feature/common/widget/button/secondary_button.dart';

import '../../../../../../wallet_app_test_widget.dart';

void main() {
  group('widgets', () {
    testWidgets('primary and secondary buttons are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ConfirmButtons(
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

    testWidgets('buttons are rendered in horizontal layout when both buttons fit on a single line', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ConfirmButtons(
          primaryButton: PrimaryButton(
            text: Text('Primary'),
          ),
          secondaryButton: SecondaryButton(
            text: Text('Secondary'),
          ),
        ),
      );

      final layoutFinder = find.byType(HorizontalConfirmButtons);
      expect(layoutFinder, findsOneWidget);
    });

    testWidgets('buttons are rendered in vertical layout when both buttons do not fit on a single line', (
      tester,
    ) async {
      await tester.pumpWidgetWithAppWrapper(
        const ConfirmButtons(
          primaryButton: PrimaryButton(
            text: Text('Primary'),
          ),
          secondaryButton: SecondaryButton(
            text: Text('Secondary'),
          ),
        ),
        textScaleSize: 3 /* large scaling so buttons dont fit on single line */,
      );

      final layoutFinder = find.byType(VerticalConfirmButtons);
      expect(layoutFinder, findsOneWidget);
    });

    testWidgets('build method should not fail when screenWidth is super narrow', (tester) async {
      // This test is introduced to verify a fix, as rendering the ConfirmButtons
      // on a very narrow screen led to negative numbers, causing the app to crash.
      await tester.pumpWidgetWithAppWrapper(
        const ConfirmButtons(
          primaryButton: PrimaryButton(
            text: Text('A'),
          ),
          secondaryButton: SecondaryButton(
            text: Text('D'),
          ),
        ),
        surfaceSize: const Size(120, 300),
      );
    });
  });
}
