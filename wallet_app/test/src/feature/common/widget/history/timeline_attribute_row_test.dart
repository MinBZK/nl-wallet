import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/timeline/interaction_timeline_attribute.dart';
import 'package:wallet/src/domain/model/timeline/operation_timeline_attribute.dart';
import 'package:wallet/src/domain/model/timeline/signing_timeline_attribute.dart';
import 'package:wallet/src/feature/common/widget/history/timeline_attribute_row.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mock_data.dart';

void main() {
  const kGoldenSize = Size(350, 115);

  group('goldens', () {
    testGoldens(
      'light timeline operation issued',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          TimelineAttributeRow(
            attribute: OperationTimelineAttribute(
              dataAttributes: [WalletMockData.textDataAttribute],
              dateTime: DateTime(2023, 1, 1),
              organization: WalletMockData.organization,
              status: OperationStatus.issued,
              card: WalletMockData.card,
            ),
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'timeline_attribute_row/light.operation.issued');
      },
    );
    testGoldens(
      'dark timeline operation issued',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          TimelineAttributeRow(
            attribute: OperationTimelineAttribute(
              dataAttributes: [WalletMockData.textDataAttribute],
              dateTime: DateTime(2023, 1, 1),
              organization: WalletMockData.organization,
              status: OperationStatus.issued,
              card: WalletMockData.card,
            ),
            onPressed: () {},
          ),
          brightness: Brightness.dark,
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'timeline_attribute_row/dark.operation.issued');
      },
    );

    testGoldens(
      'light timeline operation expired',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          TimelineAttributeRow(
            attribute: OperationTimelineAttribute(
              dataAttributes: [WalletMockData.textDataAttribute],
              dateTime: DateTime(2023, 1, 1),
              organization: WalletMockData.organization,
              status: OperationStatus.expired,
              card: WalletMockData.card,
            ),
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'timeline_attribute_row/light.operation.expired');
      },
    );
    testGoldens(
      'light timeline operation renewed',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          TimelineAttributeRow(
            attribute: OperationTimelineAttribute(
              dataAttributes: [WalletMockData.textDataAttribute],
              dateTime: DateTime(2023, 1, 1),
              organization: WalletMockData.organization,
              status: OperationStatus.renewed,
              card: WalletMockData.card,
            ),
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'timeline_attribute_row/light.operation.renewed');
      },
    );

    testGoldens(
      'light timeline interaction success',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          TimelineAttributeRow(
            attribute: WalletMockData.interactionTimelineAttribute,
            onPressed: () {},
          ),
          surfaceSize: const Size(350, 89),
        );
        await screenMatchesGolden(tester, 'timeline_attribute_row/light.interaction.success');
      },
    );

    testGoldens(
      'light timeline interaction failed',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          TimelineAttributeRow(
            attribute: WalletMockData.interactionTimelineAttribute.copyWith(status: InteractionStatus.failed),
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'timeline_attribute_row/light.interaction.failed');
      },
    );

    testGoldens(
      'light timeline interaction rejected',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          TimelineAttributeRow(
            attribute: WalletMockData.interactionTimelineAttribute.copyWith(status: InteractionStatus.rejected),
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'timeline_attribute_row/light.interaction.rejected');
      },
    );

    testGoldens(
      'light timeline signing success',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          TimelineAttributeRow(
            attribute: WalletMockData.signingTimelineAttribute,
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'timeline_attribute_row/light.signing.success');
      },
    );
    testGoldens(
      'light timeline signing rejected',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          TimelineAttributeRow(
            attribute: WalletMockData.signingTimelineAttribute.copyWith(status: SigningStatus.rejected),
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'timeline_attribute_row/light.signing.rejected');
      },
    );
  });

  group('widgets', () {
    testWidgets('onPressed is triggered', (tester) async {
      bool tapped = false;
      await tester.pumpWidgetWithAppWrapper(
        TimelineAttributeRow(
          attribute: WalletMockData.operationTimelineAttribute,
          onPressed: () => tapped = true,
        ),
      );

      // Validate that the widget exists
      final titleFinder = find.text(WalletMockData.operationTimelineAttribute.card.front.title.testValue);
      // Finds both the row title and the title in the card thumbnail
      expect(titleFinder, findsNWidgets(2));

      // Tap any title, as the whole row should be clickable
      await tester.tap(titleFinder.last);
      expect(tapped, true, reason: 'onPressed was not called');
    });
  });
}
