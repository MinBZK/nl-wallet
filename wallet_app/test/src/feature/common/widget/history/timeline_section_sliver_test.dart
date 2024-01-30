import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/timeline/interaction_timeline_attribute.dart';
import 'package:wallet/src/domain/model/timeline/operation_timeline_attribute.dart';
import 'package:wallet/src/domain/model/timeline/signing_timeline_attribute.dart';
import 'package:wallet/src/domain/model/timeline/timeline_attribute.dart';
import 'package:wallet/src/domain/model/timeline/timeline_section.dart';
import 'package:wallet/src/feature/common/widget/history/timeline_section_sliver.dart';
import 'package:wallet/src/util/extension/string_extension.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mock_data.dart';

void main() {
  const kGoldenSize = Size(350, 356);
  final mockAttributes = [
    InteractionTimelineAttribute(
      dateTime: DateTime(2023, 1, 1),
      dataAttributes: [WalletMockData.textDataAttribute],
      organization: WalletMockData.organization,
      status: InteractionStatus.success,
      policy: WalletMockData.policy,
      requestPurpose: 'Request Purpose'.untranslated,
    ),
    OperationTimelineAttribute(
        dateTime: DateTime(2023, 1, 2),
        dataAttributes: [WalletMockData.textDataAttribute],
        organization: WalletMockData.organization,
        status: OperationStatus.expired,
        cardTitle: 'Card Title'.untranslated),
    SigningTimelineAttribute(
      dateTime: DateTime(2023, 1, 3),
      dataAttributes: [WalletMockData.textDataAttribute],
      organization: WalletMockData.organization,
      status: SigningStatus.success,
      policy: WalletMockData.policy,
      document: WalletMockData.document,
    ),
  ];

  group('goldens', () {
    testGoldens(
      'light text',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          CustomScrollView(
            slivers: [
              TimelineSectionSliver(
                onRowPressed: (TimelineAttribute attribute) {},
                section: TimelineSection(
                  DateTime(2023, 1),
                  mockAttributes,
                ),
              )
            ],
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'timeline_section_sliver/light');
      },
    );
    testGoldens(
      'light text',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          CustomScrollView(
            slivers: [
              TimelineSectionSliver(
                onRowPressed: (TimelineAttribute attribute) {},
                section: TimelineSection(
                  DateTime(2023, 1),
                  mockAttributes,
                ),
              )
            ],
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.dark,
        );
        await screenMatchesGolden(tester, 'timeline_section_sliver/dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('header and items are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CustomScrollView(
          slivers: [
            TimelineSectionSliver(
              onRowPressed: (TimelineAttribute attribute) {},
              section: TimelineSection(
                DateTime(2023, 1),
                mockAttributes,
              ),
            )
          ],
        ),
      );

      // Validate that the widget exists
      final headerFinder = find.text('January 2023');
      final arribFinder1 = find.text('January 1');
      final arribFinder2 = find.text('January 2');
      final arribFinder3 = find.text('January 3');
      expect(headerFinder, findsOneWidget);
      expect(arribFinder1, findsOneWidget);
      expect(arribFinder2, findsOneWidget);
      expect(arribFinder3, findsOneWidget);
    });
  });
}
