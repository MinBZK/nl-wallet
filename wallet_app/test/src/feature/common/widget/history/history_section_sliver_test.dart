import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/event/event_section.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/feature/common/widget/history/history_section_sliver.dart';
import 'package:wallet/src/util/extension/string_extension.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mock_data.dart';
import '../../../../test_util/golden_utils.dart';

void main() {
  const kGoldenSize = Size(350, 356);
  final mockAttributes = [
    WalletEvent.disclosure(
      dateTime: DateTime(2023, 1, 1),
      status: EventStatus.success,
      relyingParty: WalletMockData.organization,
      purpose: 'disclosure'.untranslated,
      cards: [WalletMockData.card],
      type: DisclosureType.regular,
    ),
    WalletEvent.disclosure(
      dateTime: DateTime(2023, 1, 2),
      status: EventStatus.success,
      relyingParty: WalletMockData.organization,
      purpose: 'disclosure'.untranslated,
      cards: [WalletMockData.card],
      type: DisclosureType.regular,
    ),
    WalletEvent.disclosure(
      dateTime: DateTime(2023, 1, 3),
      status: EventStatus.success,
      relyingParty: WalletMockData.organization,
      purpose: 'disclosure'.untranslated,
      cards: [WalletMockData.card],
      type: DisclosureType.regular,
    ),
  ];

  group('goldens', () {
    testGoldens(
      'light text',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          CustomScrollView(
            slivers: [
              HistorySectionSliver(
                onRowPressed: (WalletEvent event) {},
                section: EventSection(
                  DateTime(2023, 1),
                  mockAttributes,
                ),
              ),
            ],
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('history_section_sliver/light');
      },
    );
    testGoldens(
      'dark text',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          CustomScrollView(
            slivers: [
              HistorySectionSliver(
                onRowPressed: (WalletEvent event) {},
                section: EventSection(
                  DateTime(2023, 1),
                  mockAttributes,
                ),
              ),
            ],
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('history_section_sliver/dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('header and items are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CustomScrollView(
          slivers: [
            HistorySectionSliver(
              onRowPressed: (WalletEvent event) {},
              section: EventSection(
                DateTime(2023, 1),
                mockAttributes,
              ),
            ),
          ],
        ),
      );

      // Validate that the widget exists
      final headerFinder = find.text('January 2023');
      final attribFinder1 = find.text('January 1');
      final attribFinder2 = find.text('January 2');
      final attribFinder3 = find.text('January 3');
      expect(headerFinder, findsOneWidget);
      expect(attribFinder1, findsOneWidget);
      expect(attribFinder2, findsOneWidget);
      expect(attribFinder3, findsOneWidget);
    });

    testWidgets('each row is exposed as a single named button, without an unnamed wrapper button', (tester) async {
      final semanticsHandle = tester.ensureSemantics();
      await tester.pumpWidgetWithAppWrapper(
        CustomScrollView(
          slivers: [
            HistorySectionSliver(
              onRowPressed: (WalletEvent event) {},
              section: EventSection(
                DateTime(2023, 1),
                mockAttributes,
              ),
            ),
          ],
        ),
      );

      final buttons = find.semantics.byFlag(SemanticsFlag.isButton).evaluate().toList();
      expect(buttons, hasLength(mockAttributes.length));
      for (final (i, button) in buttons.indexed) {
        expect(button, isSemantics(isButton: true, isFocusable: true, hasTapAction: true));
        expect(button.label, contains('January ${i + 1}'));
      }
      semanticsHandle.dispose();
    });

    testWidgets('month header is exposed as a heading', (tester) async {
      final semanticsHandle = tester.ensureSemantics();
      await tester.pumpWidgetWithAppWrapper(
        CustomScrollView(
          slivers: [
            HistorySectionSliver(
              onRowPressed: (WalletEvent event) {},
              section: EventSection(
                DateTime(2023, 1),
                mockAttributes,
              ),
            ),
          ],
        ),
      );

      expect(tester.getSemantics(find.text('January 2023')), isSemantics(label: 'January 2023', isHeader: true));
      semanticsHandle.dispose();
    });
  });
}
